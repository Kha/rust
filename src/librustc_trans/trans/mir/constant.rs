// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use back::abi;
use llvm::ValueRef;
use middle::ty::{Ty, TypeFoldable};
use rustc::middle::const_eval::{self, ConstVal};
use rustc_const_eval::ConstInt::*;
use rustc::mir::repr as mir;
use trans::common::{self, BlockAndBuilder, C_bool, C_bytes, C_floating_f64, C_integral,
                    C_str_slice, C_nil, C_undef};
use trans::consts;
use trans::expr;
use trans::inline;
use trans::type_of;
use trans::type_::Type;

use super::operand::{OperandRef, OperandValue};
use super::MirContext;


impl<'bcx, 'tcx> MirContext<'bcx, 'tcx> {
    pub fn trans_constval(&mut self,
                          bcx: &BlockAndBuilder<'bcx, 'tcx>,
                          cv: &ConstVal,
                          ty: Ty<'tcx>)
                          -> OperandRef<'tcx>
    {
        let ccx = bcx.ccx();
        let val = self.trans_constval_inner(bcx, cv, ty);
        let val = if common::type_is_immediate(ccx, ty) {
            OperandValue::Immediate(val)
        } else if common::type_is_fat_ptr(bcx.tcx(), ty) {
            let data = common::const_get_elt(ccx, val, &[abi::FAT_PTR_ADDR as u32]);
            let extra = common::const_get_elt(ccx, val, &[abi::FAT_PTR_EXTRA as u32]);
            OperandValue::FatPtr(data, extra)
        } else {
            OperandValue::Ref(val)
        };

        assert!(!ty.has_erasable_regions());

        OperandRef {
            ty: ty,
            val: val
        }
    }

    /// Translate ConstVal into a bare LLVM ValueRef.
    fn trans_constval_inner(&mut self,
                            bcx: &BlockAndBuilder<'bcx, 'tcx>,
                            cv: &ConstVal,
                            ty: Ty<'tcx>)
                            -> ValueRef
    {
        let ccx = bcx.ccx();
        let llty = type_of::type_of(ccx, ty);
        match *cv {
            ConstVal::Float(v) => C_floating_f64(v, llty),
            ConstVal::Bool(v) => C_bool(ccx, v),
            ConstVal::Integral(I8(v)) => C_integral(Type::i8(ccx), v as u64, true),
            ConstVal::Integral(I16(v)) => C_integral(Type::i16(ccx), v as u64, true),
            ConstVal::Integral(I32(v)) => C_integral(Type::i32(ccx), v as u64, true),
            ConstVal::Integral(I64(v)) => C_integral(Type::i64(ccx), v as u64, true),
            ConstVal::Integral(Isize(v)) => {
                let i = v.as_i64(ccx.tcx().sess.target.int_type);
                C_integral(Type::int(ccx), i as u64, true)
            },
            ConstVal::Integral(U8(v)) => C_integral(Type::i8(ccx), v as u64, false),
            ConstVal::Integral(U16(v)) => C_integral(Type::i16(ccx), v as u64, false),
            ConstVal::Integral(U32(v)) => C_integral(Type::i32(ccx), v as u64, false),
            ConstVal::Integral(U64(v)) => C_integral(Type::i64(ccx), v, false),
            ConstVal::Integral(Usize(v)) => {
                let u = v.as_u64(ccx.tcx().sess.target.uint_type);
                C_integral(Type::int(ccx), u, false)
            },
            ConstVal::Integral(Infer(v)) => C_integral(llty, v as u64, false),
            ConstVal::Integral(InferSigned(v)) => C_integral(llty, v as u64, true),
            ConstVal::Str(ref v) => C_str_slice(ccx, v.clone()),
            ConstVal::ByteStr(ref v) => consts::addr_of(ccx, C_bytes(ccx, v), 1, "byte_str"),
            ConstVal::Struct(id) | ConstVal::Tuple(id) |
            ConstVal::Array(id, _) | ConstVal::Repeat(id, _) => {
                let expr = bcx.tcx().map.expect_expr(id);
                bcx.with_block(|bcx| {
                    expr::trans(bcx, expr).datum.val
                })
            },
            ConstVal::Char(c) => C_integral(Type::char(ccx), c as u64, false),
            ConstVal::Dummy => unreachable!(),
            ConstVal::Function(_) => C_nil(ccx)
        }
    }

    pub fn trans_constant(&mut self,
                          bcx: &BlockAndBuilder<'bcx, 'tcx>,
                          constant: &mir::Constant<'tcx>)
                          -> OperandRef<'tcx>
    {
        let ty = bcx.monomorphize(&constant.ty);
        match constant.literal {
            mir::Literal::Item { def_id, substs } => {
                // Shortcut for zero-sized types, including function item
                // types, which would not work with lookup_const_by_id.
                if common::type_is_zero_size(bcx.ccx(), ty) {
                    let llty = type_of::type_of(bcx.ccx(), ty);
                    return OperandRef {
                        val: OperandValue::Immediate(C_undef(llty)),
                        ty: ty
                    };
                }

                let substs = bcx.tcx().mk_substs(bcx.monomorphize(&substs));
                let def_id = inline::maybe_instantiate_inline(bcx.ccx(), def_id);
                let expr = const_eval::lookup_const_by_id(bcx.tcx(), def_id, None, Some(substs))
                            .expect("def was const, but lookup_const_by_id failed").0;
                // FIXME: this is falling back to translating from HIR. This is not easy to fix,
                // because we would have somehow adapt const_eval to work on MIR rather than HIR.
                let d = bcx.with_block(|bcx| {
                    expr::trans(bcx, expr)
                });
                OperandRef::from_rvalue_datum(d.datum.to_rvalue_datum(d.bcx, "").datum)
            }
            mir::Literal::Value { ref value } => {
                self.trans_constval(bcx, value, ty)
            }
        }
    }
}
