/*
   +----------------------------------------------------------------------+
   | HipHop for PHP                                                       |
   +----------------------------------------------------------------------+
   | Copyright (c) 2010-present Facebook, Inc. (http://www.facebook.com)  |
   +----------------------------------------------------------------------+
   | This source file is subject to version 3.01 of the PHP license,      |
   | that is bundled with this package in the file LICENSE, and is        |
   | available through the world-wide-web at the following url:           |
   | http://www.php.net/license/3_01.txt                                  |
   | If you did not receive a copy of the PHP license and are unable to   |
   | obtain it through the world-wide-web, please send a note to          |
   | license@php.net so we can mail you a copy immediately.               |
   +----------------------------------------------------------------------+
*/

#pragma once

#include "hphp/runtime/base/datatype.h"
#include "hphp/runtime/vm/jit/ir-instruction.h"
#include "hphp/runtime/vm/jit/prof-data-serialize.h"
#include "hphp/runtime/vm/jit/shared-profile.h"
#include "hphp/runtime/vm/jit/type.h"

#include <folly/dynamic.h>

namespace HPHP::jit {

///////////////////////////////////////////////////////////////////////////////

/*
 * Profile the frequency of the 4 possible different behaviors of a DecRef
 * instruction.  Each execution of a DecRef must fall into exactly one of these
 * categories:
 *
 *  1) Uncounted:
 *     the type was uncounted
 *  2) Persistent:
 *     the type was refcounted and the value was persistent (so no dec happens)
 *  3) Destroyed:
 *     the type was refcounted and the value was destroyed (ie. the count was 1)
 *  4) Survived:
 *     the type was refcounted, non-persistent, but wasn't destroyed (count > 1)
 *
 */

constexpr DataType kNoDataTypesSeen = kExtraInvalidDataType;
constexpr DataType kMultipleDataTypesSeen = kInvalidDataType;

// Necessary because profile data is 0-initialized.
static_assert(static_cast<int>(kNoDataTypesSeen) == 0);

struct DecRefProfile {

  uint32_t uncounted() const {
    auto const result = int(total) - int(refcounted);
    return safe_cast<uint32_t>(std::max(result, 0));
  }

  uint32_t persistent() const {
    auto const result = int(refcounted) - int(released) - int(decremented);
    return safe_cast<uint32_t>(std::max(result, 0));
  }

  uint32_t destroyed() const {
    return released;
  }

  uint32_t survived() const {
    return decremented;
  }

  uint32_t arrayOfUncountedReleasedCount() const {
    return arrayOfUncountedReleaseCount;
  }

  float percent(uint32_t value) const {
    return total ? 100.0 * value / total : 0.0;
  }

  void serialize(ProfDataSerializer& ser) const {
    write_raw(ser, total);
    write_raw(ser, refcounted);
    write_raw(ser, released);
    write_raw(ser, decremented);
    write_raw(ser, arrayOfUncountedReleaseCount);
    write_raw(ser, datatype);
  }

  void deserialize(ProfDataDeserializer& ser) {
    read_raw(ser, total);
    read_raw(ser, refcounted);
    read_raw(ser, released);
    read_raw(ser, decremented);
    read_raw(ser, arrayOfUncountedReleaseCount);
    read_raw(ser, datatype);
  }

  /*
   * Update the profile for a dec-ref on tv.
   */
  void update(TypedValue tv);

  void updateDataType(DataType newDT);

  static void reduce(DecRefProfile& a, const DecRefProfile& b);

  folly::dynamic toDynamic() const;
  std::string toString() const;

  /*
   * The total number of times this DecRef was executed.
   */
  uint32_t total;
  /*
   * The number of times this DecRef made it at least as far as the static
   * check (meaning it was given a refcounted DataType).
   */
  uint32_t refcounted;
  /*
   * The number of times this DecRef went to zero and called the release method.
   */
  uint32_t released;
  /*
   * The number of times this DecRef actually decremented the count (meaning it
   * got a non-persistent, refcounted value with count > 1).
   */
  uint32_t decremented;

  /*
   * The number of times an array of uncounted elements was released.
   */
  uint32_t arrayOfUncountedReleaseCount;

  /*
   * 'datatype' is populated with a unique datatype, if we see one, during
   * profiling. It is initialized to 0 (kNoDataTypesSeen) before any datatypes
   * are seen. If non-unique datatypes are seen, it is set to
   * kMultipleDataTypesSeen.
   */
  DataType datatype;

  /*
   * To ensure that `update` is constant time, we examine at most this many
   * values when checking when the input is an array of uncounted values.
   * Arrays with a large number of values are usually monotyped, so we don't
   * get much benefit from examining more.
   */
  static constexpr size_t kMaxNumProfiledElements = 16;
};

const StringData* decRefProfileKey(int locId);
const StringData* decRefProfileKey(const IRInstruction* inst);

SharedProfile<DecRefProfile> decRefProfile(
    const TransContext& context, const IRInstruction* inst);

SharedProfile<DecRefProfile> decRefProfile(
    const TransContext& context, const BCMarker& marker, int locId);

///////////////////////////////////////////////////////////////////////////////

}
