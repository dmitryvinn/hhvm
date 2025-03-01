// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the "hack" directory of this source tree.

// Interestingly, HHAS does not represent the declared types of constants,
// unlike formal parameters and return types. We might consider fixing this.
// Also interestingly, abstract constants are not emitted at all.

use env::{emitter::Emitter, Env};
use ffi::Maybe;
use instruction_sequence::{InstrSeq, Result};
use oxidized::ast;
use runtime::TypedValue;

#[derive(Debug)]
#[repr(C)]
pub struct HhasConstant<'arena> {
    pub name: hhbc_id::r#const::ConstType<'arena>,
    pub value: Maybe<TypedValue<'arena>>,
    pub initializer_instrs: Maybe<InstrSeq<'arena>>,
    pub is_abstract: bool,
}

pub fn from_ast<'a, 'arena, 'decl>(
    emitter: &mut Emitter<'arena, 'decl>,
    env: &Env<'a, 'arena>,
    id: &'a ast::Id,
    is_abstract: bool,
    expr: Option<&ast::Expr>,
) -> Result<HhasConstant<'arena>> {
    let alloc = env.arena;
    let (value, initializer_instrs) = match expr {
        None => (None, None),
        Some(init) => match ast_constant_folder::expr_to_typed_value(emitter, init) {
            Ok(v) => (Some(v), None),
            Err(_) => (
                Some(TypedValue::Uninit),
                Some(emit_expression::emit_expr(emitter, env, init)?),
            ),
        },
    };
    Ok(HhasConstant {
        name: hhbc_id::r#const::ConstType::from_ast_name(alloc, id.name()),
        value: Maybe::from(value),
        initializer_instrs: Maybe::from(initializer_instrs),
        is_abstract,
    })
}
