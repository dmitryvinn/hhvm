// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the "hack" directory of this source tree.

use emit_expression::{emit_reified_arg, is_reified_tparam};
use env::{emitter::Emitter, Env};
use instruction_sequence::*;
use naming_special_names_rust as sn;
use oxidized::{aast, ast_defs::Id, file_info, pos::Pos};

use hash::HashSet;
use oxidized_by_ref::shallow_decl_defs;

#[derive(Debug, Clone)]
pub enum ReificationLevel {
    /// There is a reified generic
    Definitely,
    /// There is no function or class reified generics, but there may be an inferred one
    Maybe,
    Not,
    Unconstrained,
}
impl ReificationLevel {
    fn combine(v1: &Self, v2: &Self) -> Self {
        match (v1, v2) {
            (Self::Definitely, _) | (_, Self::Definitely) => Self::Definitely,
            (Self::Maybe, _) | (_, Self::Maybe) => Self::Maybe,
            _ => Self::Not,
        }
    }
}

pub(crate) fn get_erased_tparams<'a, 'arena>(
    env: &'a Env<'a, 'arena>,
) -> impl Iterator<Item = String> + 'a {
    env.scope.get_tparams().into_iter().filter_map(|tp| {
        if tp.reified != aast::ReifyKind::Reified {
            Some(tp.name.1.clone()) // TODO(hrust) figure out how to return &str
        } else {
            None
        }
    })
}

pub(crate) fn has_reified_type_constraint<'a, 'arena>(
    env: &Env<'a, 'arena>,
    h: &aast::Hint,
) -> ReificationLevel {
    use aast::Hint_;
    fn is_all_erased<'a>(
        env: &'a Env<'_, '_>,
        mut h_iter: impl Iterator<Item = &'a aast::Hint>,
    ) -> bool {
        let erased_tparams: HashSet<String> = get_erased_tparams(env).collect();
        h_iter.all(|h| {
            if let Hint_::Happly(Id(_, ref id), ref apply_hints) = *h.1 {
                if apply_hints.is_empty() {
                    return id == "_" || erased_tparams.contains(id);
                }
            }
            false
        })
    }
    match &*h.1 {
        Hint_::Happly(Id(_, id), hs) => {
            if is_reified_tparam(env, true, id).is_some()
                || is_reified_tparam(env, false, id).is_some()
            {
                ReificationLevel::Definitely
            } else if hs.is_empty() || is_all_erased(env, hs.iter()) {
                ReificationLevel::Not
            } else {
                hs.iter().rev().fold(ReificationLevel::Maybe, |v, h| {
                    ReificationLevel::combine(&v, &has_reified_type_constraint(env, h))
                })
            }
        }
        Hint_::Hsoft(h) | Hint_::Hlike(h) | Hint_::Hoption(h) => {
            has_reified_type_constraint(env, h)
        }
        Hint_::Hprim(_)
        | Hint_::Hmixed
        | Hint_::Hnonnull
        | Hint_::HvecOrDict(_, _)
        | Hint_::Hthis
        | Hint_::Hnothing
        | Hint_::Hdynamic
        | Hint_::Htuple(_)
        | Hint_::Hunion(_)
        | Hint_::Hintersection(_)
        | Hint_::Hshape(_)
        | Hint_::Hfun(_)
        | Hint_::Haccess(_, _)
        | Hint_::HfunContext(_)
        | Hint_::Hvar(_) => ReificationLevel::Not,
        // Not found in the original AST
        Hint_::Herr | Hint_::Hany => panic!("Should be a naming error"),
        Hint_::Habstr(_, _) => panic!("TODO Unimplemented: Not in the original AST"),
    }
}

fn remove_awaitable(aast::Hint(pos, hint): aast::Hint) -> aast::Hint {
    use aast::{Hint, Hint_};
    match *hint {
        Hint_::Happly(sid, mut hs)
            if hs.len() == 1 && sid.1.eq_ignore_ascii_case(sn::classes::AWAITABLE) =>
        {
            hs.pop().unwrap()
        }
        // For @Awaitable<T>, the soft type hint is moved to the inner type, i.e @T
        Hint_::Hsoft(h) => Hint(pos, Box::new(Hint_::Hsoft(remove_awaitable(h)))),
        // For ~Awaitable<T>, the like-type  hint is moved to the inner type, i.e ~T
        Hint_::Hlike(h) => Hint(pos, Box::new(Hint_::Hlike(remove_awaitable(h)))),
        // For ?Awaitable<T>, the optional is dropped
        Hint_::Hoption(h) => remove_awaitable(h),
        Hint_::Htuple(_)
        | Hint_::Hunion(_)
        | Hint_::Hintersection(_)
        | Hint_::Hshape(_)
        | Hint_::Hfun(_)
        | Hint_::Haccess(_, _)
        | Hint_::Happly(_, _)
        | Hint_::HfunContext(_)
        | Hint_::Hvar(_) => Hint(pos, hint),
        Hint_::Herr
        | Hint_::Hany
        | Hint_::Hmixed
        | Hint_::Hnonnull
        | Hint_::Habstr(_, _)
        | Hint_::HvecOrDict(_, _)
        | Hint_::Hprim(_)
        | Hint_::Hthis
        | Hint_::Hnothing
        | Hint_::Hdynamic => panic!("TODO Unimplemented Did not exist on legacy AST"),
    }
}

pub(crate) fn convert_awaitable<'a, 'arena>(env: &Env<'a, 'arena>, h: aast::Hint) -> aast::Hint {
    if env.scope.is_in_async() {
        remove_awaitable(h)
    } else {
        h
    }
}

pub(crate) fn simplify_verify_type<'a, 'arena, 'decl>(
    e: &mut Emitter<'arena, 'decl>,
    env: &mut Env<'a, 'arena>,
    pos: &Pos,
    check: InstrSeq<'arena>,
    hint: &aast::Hint,
    verify_instr: InstrSeq<'arena>,
) -> Result<InstrSeq<'arena>> {
    let alloc = env.arena;
    let get_ts = |e, hint| Ok(emit_reified_arg(e, env, pos, false, hint)?.0);
    let aast::Hint(_, hint_) = hint;
    if let aast::Hint_::Hoption(ref hint) = **hint_ {
        let label_gen = e.label_gen_mut();
        let done_label = label_gen.next_regular();
        Ok(InstrSeq::gather(
            alloc,
            vec![
                check,
                instr::jmpnz(alloc, done_label),
                get_ts(e, hint)?,
                verify_instr,
                instr::label(alloc, done_label),
            ],
        ))
    } else {
        Ok(InstrSeq::gather(
            alloc,
            vec![get_ts(e, hint)?, verify_instr],
        ))
    }
}

pub(crate) fn remove_erased_generics<'a, 'arena>(
    env: &Env<'a, 'arena>,
    h: aast::Hint,
) -> aast::Hint {
    use aast::{Hint, Hint_, NastShapeInfo, ShapeFieldInfo};
    fn rec<'a, 'arena>(env: &Env<'a, 'arena>, Hint(pos, h_): Hint) -> Hint {
        fn modify<'a, 'arena>(env: &Env<'a, 'arena>, id: String) -> String {
            if get_erased_tparams(env).any(|p| p == id) {
                "_".into()
            } else {
                id
            }
        }
        let h_ = match *h_ {
            Hint_::Happly(Id(pos, id), hs) => Hint_::Happly(
                Id(pos, modify(env, id)),
                hs.into_iter().map(|h| rec(env, h)).collect(),
            ),
            Hint_::Hsoft(h) => Hint_::Hsoft(rec(env, h)),
            Hint_::Hlike(h) => Hint_::Hlike(rec(env, h)),
            Hint_::Hoption(h) => Hint_::Hoption(rec(env, h)),
            Hint_::Htuple(hs) => Hint_::Htuple(hs.into_iter().map(|h| rec(env, h)).collect()),
            Hint_::Hunion(hs) => Hint_::Hunion(hs.into_iter().map(|h| rec(env, h)).collect()),
            Hint_::Hintersection(hs) => {
                Hint_::Hintersection(hs.into_iter().map(|h| rec(env, h)).collect())
            }
            Hint_::Hshape(NastShapeInfo {
                allows_unknown_fields,
                field_map,
            }) => {
                let field_map = field_map
                    .into_iter()
                    .map(|sfi: ShapeFieldInfo| ShapeFieldInfo {
                        hint: rec(env, sfi.hint),
                        ..sfi
                    })
                    .collect();
                Hint_::Hshape(NastShapeInfo {
                    allows_unknown_fields,
                    field_map,
                })
            }
            h_ @ Hint_::Hfun(_) | h_ @ Hint_::Haccess(_, _) => h_,
            Hint_::Herr
            | Hint_::Hany
            | Hint_::Hmixed
            | Hint_::Hnonnull
            | Hint_::Habstr(_, _)
            | Hint_::HvecOrDict(_, _)
            | Hint_::Hprim(_)
            | Hint_::Hthis
            | Hint_::Hnothing
            | Hint_::Hdynamic => panic!("TODO Unimplemented Did not exist on legacy AST"),
            Hint_::HfunContext(_) | Hint_::Hvar(_) => {
                panic!("Coeffects are currently erased during compilation")
            }
        };
        Hint(pos, Box::new(h_))
    }
    rec(env, h)
}

/// Warning: Experimental usage of decls in compilation.
/// Given a hint, if the hint is an Happly(id, _), checks if the id is a class
/// that has reified generics.
pub(crate) fn happly_decl_has_no_reified_generics<'arena, 'decl>(
    emitter: &mut Emitter<'arena, 'decl>,
    aast::Hint(_, hint): &aast::Hint,
) -> bool {
    use aast::{Hint_, ReifyKind};
    match hint.as_ref() {
        Hint_::Happly(Id(_, id), _) => {
            if let Ok(shallow_decl_defs::Decl::Class(class_decl)) =
                emitter.get_decl(file_info::NameType::Class, id)
            {
                if class_decl
                    .tparams
                    .iter()
                    .all(|tparam| -> bool { tparam.reified == ReifyKind::Erased })
                {
                    return true;
                }
            }
            false
        }
        Hint_::Hoption(_)
        | Hint_::Hlike(_)
        | Hint_::Hfun(_)
        | Hint_::Htuple(_)
        | Hint_::Hshape(_)
        | Hint_::Haccess(_, _)
        | Hint_::Hsoft(_) => false,
        // The rest are not found on the AST at this stage
        _ => false,
    }
}
