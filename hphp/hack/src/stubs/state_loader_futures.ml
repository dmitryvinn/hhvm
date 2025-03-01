(*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the "hack" directory of this source tree.
 *
 *)

let load
    ~env:_
    ~progress_callback:_
    ~watchman_opts:_
    ~ignore_hh_version:_
    ~saved_state_type:_ =
  Future.of_value (Error "Not implemented")

let wait_for_finish _ = failwith "Not implemented"

let wait_for_finish_with_debug_details _ = failwith "Not implemented"
