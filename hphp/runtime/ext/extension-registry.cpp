#include "hphp/runtime/ext/extension-registry.h"

#include <sstream>

#include "hphp/runtime/base/array-init.h"
#include "hphp/runtime/base/file-util.h"
#include "hphp/runtime/base/runtime-option.h"
#include "hphp/runtime/version.h"
#include "hphp/runtime/vm/jit/prof-data-serialize.h"
#include "hphp/runtime/vm/litstr-table.h"
#include "hphp/system/systemlib.h"

#ifdef HAVE_LIBDL
# include <dlfcn.h>
# ifndef RTLD_LAZY
#  define RTLD_LAZY 1
# endif
# ifndef RTLD_GLOBAL
#  define RTLD_GLOBAL 0
# endif
# if defined(RTLD_GROUP) && defined(RTLD_WORLD) && defined(RTLD_PARENT)
#  define DLOPEN_FLAGS (RTLD_LAZY|RTLD_GLOBAL|RTLD_GROUP|RTLD_WORLD|RTLD_PARENT)
# else
#  define DLOPEN_FLAGS (RTLD_LAZY|RTLD_GLOBAL)
# endif
#endif

namespace HPHP::ExtensionRegistry {
/////////////////////////////////////////////////////////////////////////////

// s_exts isn't necessarily initialized by the time Extensions
// start registering themselves, so we have to be explicit about
// allocating/initializing/destroying this rather than just
// putting it global and letting the compiler deal with it. :(
typedef std::map<std::string, Extension*, stdltistr> ExtensionMap;
static ExtensionMap *s_exts = nullptr;

// just to make valgrind cleaner
struct ExtensionRegistryUninitializer {
  ~ExtensionRegistryUninitializer() {
    delete s_exts;
  }
};
static ExtensionRegistryUninitializer s_extension_registry_uninitializer;

typedef std::vector<Extension*> OrderedExtensionVector;
static OrderedExtensionVector s_ordered;

static bool s_sorted = false;
static bool s_initialized = false;

static void sortDependencies();

///////////////////////////////////////////////////////////////////////////////
// dlfcn wrappers

static void* dlopen(ATTRIBUTE_UNUSED const char* dso) {
#ifdef HAVE_LIBDL
  return ::dlopen(dso, DLOPEN_FLAGS);
#else
  return nullptr;
#endif
}

static void*
dlsym(ATTRIBUTE_UNUSED void* mod, ATTRIBUTE_UNUSED const char* sym) {
#ifdef HAVE_LIBDL
# ifdef LIBDL_NEEDS_UNDERSCORE
  std::string tmp("_");
  tmp += sym;
  sym = tmp.c_str();
# endif
  return ::dlsym(mod, sym);
#else
  return nullptr;
#endif
}

static const char* dlerror() {
#ifdef HAVE_LIBDL
  return ::dlerror();
#else
  return "Your system does not support dlopen()";
#endif
}

/////////////////////////////////////////////////////////////////////////////
// Registration API

void registerExtension(Extension* ext) {
  if (!s_exts) {
    // Not really thread-safe,
    // but in practice this happens at process startup
    // before any threads are spun up.
    s_exts = new ExtensionMap;
  }
  const auto& name = ext->getName();
  assertx(s_exts->find(name) == s_exts->end());
  (*s_exts)[name] = ext;
}

bool isLoaded(const char* name, bool enabled_only /*= true */) {
  assertx(s_exts);
  auto it = s_exts->find(name);
  return (it != s_exts->end()) &&
         (!enabled_only || it->second->moduleEnabled());
}

Extension* get(const char* name, bool enabled_only /*= true */) {
  assertx(s_exts);
  auto it = s_exts->find(name);
  if ((it != s_exts->end()) &&
      (!enabled_only || it->second->moduleEnabled())) {
    return it->second;
  }
  return nullptr;
}

Array getLoaded(bool enabled_only /*= true */) {
  assertx(s_exts);
  // Overestimate.
  VecInit ret(s_exts->size());
  for (auto& kv : (*s_exts)) {
    if (!enabled_only || kv.second->moduleEnabled()) {
      ret.append(String(kv.second->getName()));
    }
  }
  return ret.toArray();
}

/////////////////////////////////////////////////////////////////////////////
// Lifecycle delegators

static void moduleLoad(const std::string& extpath) {
  // Extensions are self-registering,
  // so we bring in the SO then
  // throw away its handle.
  void *ptr = dlopen(extpath.c_str());
  if (!ptr) {
    throw Exception("Could not open extension %s: %s",
                    extpath.c_str(), dlerror());
  }
  auto getModuleBuildInfo =
    (ExtensionBuildInfo *(*)())dlsym(ptr, "getModuleBuildInfo");
  if (!getModuleBuildInfo) {
    throw Exception("Could not load extension %s: %s (%s)",
                    extpath.c_str(),
                    "getModuleBuildInfo() symbol not defined.",
                    dlerror());
  }
  auto info = getModuleBuildInfo();
  if (info->dso_version != HHVM_DSO_VERSION) {
    throw Exception("%s was built with an incompatible DSO API. "
                    "Expected %ld, got %ld",
                    extpath.c_str(),
                    (long)HHVM_DSO_VERSION,
                    (long)info->dso_version);
  }
  if (info->branch_id != HHVM_VERSION_BRANCH) {
    throw Exception("%s was built for HHVM %d.%d, "
                    "and cannot be loaded with HHVM %d.%d",
                    extpath.c_str(),
                    (int)(info->branch_id >> 16),
                    (int)((info->branch_id >> 8) & 0xFF),
                    (int)HHVM_VERSION_MAJOR,
                    (int)HHVM_VERSION_MINOR);
  }
  auto getModule = (Extension *(*)())dlsym(ptr, "getModule");
  if (!getModule) {
    throw Exception("Could not load extension %s: %s (%s)",
                    extpath.c_str(),
                    "getModule() symbol not defined.",
                    dlerror());
  }
  getModule()->setDSOName(extpath);
}

void moduleLoad(const IniSetting::Map& ini, Hdf hdf) {
  std::set<std::string> extFiles;

  // Load up any dynamic extensions from extension_dir
  std::string extDir = RuntimeOption::ExtensionDir;
  for (auto& extLoc : RuntimeOption::Extensions) {
    if (extLoc.empty()) {
      continue;
    }
    if (!FileUtil::isAbsolutePath(extLoc)) {
      if (extDir == "") {
        continue;
      }
      extLoc = extDir + "/" + extLoc;
    }

    extFiles.insert(extLoc);
  }

  // Load up any dynamic extensions from dynamic extensions options
  for (auto& extLoc : RuntimeOption::DynamicExtensions) {
    if (extLoc.empty()) {
      continue;
    }
    if (!FileUtil::isAbsolutePath(extLoc)) {
      extLoc = RuntimeOption::DynamicExtensionPath + "/" + extLoc;
    }

    extFiles.insert(extLoc);
  }

  for (std::string extFile : extFiles) {
    moduleLoad(extFile);
  }

  // Invoke Extension::moduleLoad() callbacks
  if (extFiles.size() > 0 || !s_sorted) {
    sortDependencies();
  }
  assertx(s_sorted);

  for (auto& ext : s_ordered) {
    ext->moduleLoad(ini, hdf);
  }
}

void moduleInit() {
  bool wasInited = SystemLib::s_inited;
  auto const wasDB = RuntimeOption::EvalDumpBytecode;
  RuntimeOption::EvalDumpBytecode &= ~1;
  SCOPE_EXIT {
    SystemLib::s_inited = wasInited;
    RuntimeOption::EvalDumpBytecode = wasDB;
  };
  SystemLib::s_inited = false;
  assertx(s_sorted);
  for (auto& ext : s_ordered) {
    ext->moduleInit();
  }
  s_initialized = true;
}

void cliClientInit() {
  assertx(s_sorted);
  for (auto& ext : s_ordered) {
    ext->cliClientInit();
  }
  s_initialized = true;
}

void moduleShutdown() {
  assertx(s_exts);
  assertx(s_sorted);
  for (auto it = s_ordered.rbegin();
       it != s_ordered.rend(); ++it) {
    (*it)->moduleShutdown();
  }
  s_exts->clear();
  s_ordered.clear();
  s_sorted = false;
  s_initialized = false;
}

void threadInit() {
  // This can actually happen both before and after LoadModules()
  if (!s_sorted) sortDependencies();
  assertx(s_sorted);
  for (auto& ext : s_ordered) {
    ext->threadInit();
  }
}

void threadShutdown() {
  assertx(s_sorted);
  for (auto it = s_ordered.rbegin();
       it != s_ordered.rend(); ++it) {
    (*it)->threadShutdown();
  }
}

void requestInit() {
  assertx(s_sorted);
  for (auto& ext : s_ordered) {
    ext->requestInit();
  }
}

void requestShutdown() {
  assertx(s_sorted);
  for (auto it = s_ordered.rbegin();
       it != s_ordered.rend(); ++it) {
    (*it)->requestShutdown();
  }
}

bool modulesInitialised() { return s_initialized; }

void serialize(jit::ProfDataSerializer& ser) {
  std::vector<std::pair<std::string, std::string>> extData;
  for (auto& ext : *s_exts) {
    auto name = ext.first;
    auto data = ext.second->serialize();
    if (!data.size()) continue;
    extData.push_back({std::move(name), std::move(data)});
  }
  uint32_t len = extData.size();
  jit::write_raw<uint32_t>(ser, len);
  for (const auto& ext : extData) {
    len = ext.first.size();
    jit::write_raw<uint32_t>(ser, len);
    jit::write_raw(ser, ext.first.c_str(), len);
    len = ext.second.size();
    jit::write_raw<uint32_t>(ser, len);
    jit::write_raw(ser, ext.second.c_str(), len);
  }
}

void deserialize(jit::ProfDataDeserializer& des) {
  auto const nExts = jit::read_raw<uint32_t>(des);
  for (uint32_t i = 0; i < nExts; ++i) {
    uint32_t len = jit::read_raw<uint32_t>(des);
    std::string str;
    str.resize(len);
    jit::read_raw(des, str.data(), len);
    auto ext = get(str.data(), false);
    if (!ext) throw std::runtime_error{str.c_str()};
    len = jit::read_raw<uint32_t>(des);
    str.resize(len);
    jit::read_raw(des, str.data(), len);
    ext->deserialize(std::move(str));
  }
}

/////////////////////////////////////////////////////////////////////////////

static
Extension* findResolvedExt(const Extension::DependencySetMap& unresolved,
                           const Extension::DependencySet& resolved) {
  if (unresolved.empty()) return nullptr;

  for (auto& ed : unresolved) {
    Extension* ret = ed.first;
    for (auto& req : ed.second) {
      if (resolved.find(req) == resolved.end()) {
        // Something we depend on still isn't resolved, try another
        ret = nullptr;
        break;
      }
    }
    if (ret) return ret;
  }
  return nullptr;
}

static void sortDependencies() {
  assertx(s_exts);
  s_ordered.clear();

  Extension::DependencySet resolved;
  Extension::DependencySetMap unresolved;

  // First pass, identify the easy(common) case of modules
  // with no dependencies and put that at the front of the list
  // defer all other for slower resolution
  for (auto& kv : (*s_exts)) {
    auto ext = kv.second;
    auto deps = ext->getDeps();
    if (deps.empty()) {
      s_ordered.push_back(ext);
      resolved.insert(kv.first);
      continue;
    }
    unresolved[ext] = deps;
  }

  // Second pass, check each remaining extension against
  // their dependency list until they have all been loaded
  while (auto ext = findResolvedExt(unresolved, resolved)) {
    s_ordered.push_back(ext);
    resolved.insert(ext->getName());
    unresolved.erase(ext);
  }

  if (UNLIKELY(!unresolved.empty())) {
    // Alerts user to cirular dependency in extensions
    // e.g. Unable to resovle dependencies for extension(s):
    //         A(depends: B) B(depends: C) C(depends: A)

    std::stringstream ss;
    ss << "Unable to resolve dependencies for extension(s):";
    for (auto& kv : unresolved) {
      ss << " " << kv.first->getName() << "(depends:";
      for (auto& req : kv.second) {
        ss << " " << req;
      }
      ss << ")";
    }
    throw Exception(ss.str());
  }

  assertx(s_ordered.size() == s_exts->size());
  s_sorted = true;
}

/////////////////////////////////////////////////////////////////////////////
} // namespace HPHP::ExtensionRegistry
