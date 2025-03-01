/*
   +----------------------------------------------------------------------+
   | HipHop for PHP                                                       |
   +----------------------------------------------------------------------+
   | Copyright (c) 2010-present Facebook, Inc. (http://www.facebook.com)  |
   +----------------------------------------------------------------------+
   | This source path is subject to version 3.01 of the PHP license,      |
   | that is bundled with this package in the path LICENSE, and is        |
   | available through the world-wide-web at the following url:           |
   | http://www.php.net/license/3_01.txt                                  |
   | If you did not receive a copy of the PHP license and are unable to   |
   | obtain it through the world-wide-web, please send a note to          |
   | license@php.net so we can mail you a copy immediately.               |
   +----------------------------------------------------------------------+
*/

#pragma once

#include <memory>

#include <folly/dynamic.h>
#include <folly/experimental/io/FsUtil.h>

#include "hphp/runtime/base/autoload-map.h"
#include "hphp/runtime/ext/facts/autoload-db.h"
#include "hphp/runtime/ext/facts/watcher.h"

namespace HPHP {
namespace Facts {

/**
 * Create a FactsStore that learns about changed files from its Watcher and
 * accordingly updates the DB that `dbHandle` returns.
 */
std::shared_ptr<FactsStore> make_watcher_facts(
    folly::fs::path root,
    AutoloadDB::Handle dbHandle,
    std::unique_ptr<Watcher> watcher,
    bool shouldSubscribe,
    std::vector<std::string> indexedMethodAttributes);

/**
 * Create a FactsStore that trusts the DB that `dbHandle` returns, and never
 * modifies it.
 */
std::shared_ptr<FactsStore>
make_trusted_facts(folly::fs::path root, AutoloadDB::Handle dbHandle);

} // namespace Facts
} // namespace HPHP
