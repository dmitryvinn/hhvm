(*
 * Copyright (c) 2018, Facebook, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the "hack" directory of this source tree.
 *
 *)

(* Eager Initialization:

   hh_server can initialize either by typechecking the entire project (aka
   starting from a "fresh state") or by loading from a saved state and
   typechecking what has changed. Historically, eager initialization was able to
   do both, but we had little need for eager init from saved state, and removed
   it.

   If we perform an eager init from a fresh state, we run the following phases:

     Parsing -> Naming -> Type-decl -> Type-check

   If we are initializing lazily from a saved state, we do

     Run load script and parsing concurrently -> Naming -> Type-check

   Then we typecheck only the files that have changed since the state was saved.
   Type-decl is performed lazily as needed.
*)

open Hh_prelude
open SearchServiceRunner
open ServerEnv
open ServerInitTypes
module SLC = ServerLocalConfig

let type_decl
    (genv : ServerEnv.genv)
    (env : ServerEnv.env)
    (fast : Naming_table.fast)
    (t : float) : ServerEnv.env * float =
  ServerProgress.send_progress "evaluating type declarations";
  let bucket_size = genv.local_config.SLC.type_decl_bucket_size in
  let ctx = Provider_utils.ctx_from_server_env env in
  let errorl = Decl_service.go ~bucket_size ctx genv.workers fast in
  Stats.(stats.init_heap_size <- SharedMem.SMTelemetry.heap_size ());
  HackEventLogger.type_decl_end t;
  let t = Hh_logger.log_duration "Type-decl" t in
  let env = { env with errorl = Errors.merge errorl env.errorl } in
  (env, t)

let init
    (genv : ServerEnv.genv)
    (lazy_level : lazy_level)
    (env : ServerEnv.env)
    (cgroup_steps : CgroupProfiler.step_group) : ServerEnv.env * float =
  let init_telemetry =
    ServerEnv.Init_telemetry.make
      ServerEnv.Init_telemetry.Init_eager
      (Telemetry.create ()
      |> Telemetry.float_ ~key:"start_time" ~value:(Unix.gettimeofday ()))
  in

  (* We don't support a saved state for eager init. *)
  let (get_next, t) =
    ServerInitCommon.indexing ~telemetry_label:"eager.init.indexing" genv
  in
  let lazy_parse =
    match lazy_level with
    | Parse -> true
    | _ -> false
  in
  (* Parsing entire repo, too many files to trace *)
  let trace = false in
  let (env, t) =
    ServerInitCommon.parsing
      ~lazy_parse
      genv
      env
      ~get_next
      t
      ~trace
      ~cache_decls:true
      ~telemetry_label:"eager.init.parsing"
      ~cgroup_steps
      ~worker_call:MultiWorker.wrapper
  in
  if not (ServerArgs.check_mode genv.options) then
    SearchServiceRunner.update_fileinfo_map
      env.naming_table
      ~source:SearchUtils.Init;
  let (env, t) =
    ServerInitCommon.naming
      env
      t
      ~telemetry_label:"eager.init.naming"
      ~cgroup_steps
  in
  let fast = Naming_table.to_fast env.naming_table in
  let failed_parsing = Errors.get_failed_files env.errorl Errors.Parsing in
  let fast =
    Relative_path.Set.fold
      failed_parsing
      ~f:(fun x m -> Relative_path.Map.remove m x)
      ~init:fast
  in
  let (env, t) =
    match lazy_level with
    | Off -> type_decl genv env fast t
    | _ -> (env, t)
  in
  (* Type-checking everything *)
  ServerInitCommon.type_check
    genv
    env
    (Relative_path.Map.keys fast)
    init_telemetry
    t
    ~telemetry_label:"eager.init.type_check"
    ~cgroup_steps
