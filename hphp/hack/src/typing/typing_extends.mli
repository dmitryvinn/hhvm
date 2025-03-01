(*
 * Copyright (c) 2015, Facebook, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the "hack" directory of this source tree.
 *
 *)

(** Checks that a classish type implements its interfaces, extends its base class, and
    uses its traits.
    [implements] is the list of interfaces the classish type directly implements.
    [parents] is the list of direct ancestors and traits the class directly uses. *)
val check_implements_extends_uses :
  Typing_env_types.env ->
  (* All directly implemented interfaces *)
  implements:Typing_defs.decl_ty list ->
  (* All direct parents (interfaces, base type, traits) *)
  parents:Typing_defs.decl_ty list ->
  (* The type to be checked *)
  Pos.t * Decl_provider.Class.t ->
  Typing_env_types.env
