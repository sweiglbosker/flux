use std::sync::OnceLock;

use flux_arc_interner::List;
use flux_common::bug;
use flux_syntax::symbols::sym;
use rustc_attr_ir::LangItem;
use rustc_data_structures::unord::UnordMap;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{self as rustc_ty, TypingEnv};
use rustc_span::{DUMMY_SP, Symbol};

use crate::{
    def_id::FluxDefId,
    global_env::GlobalEnv,
    rty::{self, AliasReft, AssocRefinements, AssocReft, BaseTy, BinOp, Expr, GenericArg, Sort},
};

impl<'tcx> GlobalEnv<'_, 'tcx> {
    #[allow(
        clippy::disallowed_methods,
        reason = "this file is the source of truth for builtin assoc refts def ids"
    )]
    pub fn builtin_assoc_refts(self, def_id: DefId) -> Option<AssocRefinements> {
        static BUILTIN: OnceLock<UnordMap<DefId, AssocRefinements>> = OnceLock::new();

        BUILTIN
            .get_or_init(|| {
                let tcx = self.tcx();

                let mut map = UnordMap::default();

                // FnOnce
                let fn_once_id = tcx.require_lang_item(LangItem::FnOnce, DUMMY_SP);
                map.insert(
                    fn_once_id,
                    AssocRefinements {
                        items: List::from_arr([AssocReft::new(
                            FluxDefId::new(fn_once_id, sym::no_panic),
                            false,
                            tcx.def_span(fn_once_id),
                        )]),
                    },
                );

                // Sized
                let sized_id = tcx.require_lang_item(LangItem::Sized, DUMMY_SP);
                map.insert(
                    sized_id,
                    AssocRefinements {
                        items: List::from_arr([
                            AssocReft::new(
                                FluxDefId::new(sized_id, sym::size_of),
                                false,
                                tcx.def_span(sized_id),
                            ),
                            AssocReft::new(
                                FluxDefId::new(sized_id, sym::align_of),
                                false,
                                tcx.def_span(sized_id),
                            ),
                        ]),
                    },
                );

                // MetaSized
                let meta_sized_id = tcx.require_lang_item(LangItem::MetaSized, DUMMY_SP);
                map.insert(
                    meta_sized_id,
                    AssocRefinements {
                        items: List::from_arr([
                            AssocReft::new(
                                FluxDefId::new(meta_sized_id, sym::size_of_val),
                                false,
                                tcx.def_span(meta_sized_id),
                            ),
                            AssocReft::new(
                                FluxDefId::new(meta_sized_id, sym::align_of_val),
                                false,
                                tcx.def_span(meta_sized_id),
                            ),
                        ]),
                    },
                );
                map
            })
            .get(&def_id)
            .cloned()
    }

    #[allow(
        clippy::disallowed_methods,
        reason = "this file is the source of truth for builtin assoc refts def ids"
    )]
    pub fn builtin_assoc_reft_sort(
        self,
        assoc_id: FluxDefId,
    ) -> Option<rty::EarlyBinder<rty::FuncSort>> {
        static BUILTIN: OnceLock<UnordMap<FluxDefId, rty::FuncSort>> = OnceLock::new();

        BUILTIN
            .get_or_init(|| {
                let tcx = self.tcx();

                let mut map = UnordMap::default();

                // FnOnce
                let fn_once_id = tcx.require_lang_item(LangItem::FnOnce, DUMMY_SP);
                map.insert(
                    FluxDefId::new(fn_once_id, sym::no_panic),
                    rty::FuncSort::new(vec![], rty::Sort::Bool),
                );

                // Sized
                let sized_id = tcx.require_lang_item(LangItem::Sized, DUMMY_SP);
                map.insert(
                    FluxDefId::new(sized_id, sym::size_of),
                    rty::FuncSort::new(vec![], rty::Sort::Int),
                );
                map.insert(
                    FluxDefId::new(sized_id, sym::align_of),
                    rty::FuncSort::new(vec![], rty::Sort::Int),
                );

                // MetaSized: both take the pointer metadata as an argument
                let meta_sized_id = tcx.require_lang_item(LangItem::MetaSized, DUMMY_SP);
                map.insert(
                    FluxDefId::new(meta_sized_id, sym::size_of_val),
                    rty::FuncSort::new(vec![rty::Sort::Int], rty::Sort::Int),
                );
                map.insert(
                    FluxDefId::new(meta_sized_id, sym::align_of_val),
                    rty::FuncSort::new(vec![rty::Sort::Int], rty::Sort::Int),
                );
                map
            })
            .get(&assoc_id)
            .cloned()
            .map(rty::EarlyBinder)
    }

    /// Returns the [`FluxDefId`] for a built-in associated refinement given the parent trait's
    /// [`DefId`] and the assoc reft [`Symbol`]. Panics if no such built-in assoc reft exists.
    ///
    /// This is the single source of truth for looking up built-in assoc reft ids; callers should
    /// use this instead of constructing a [`FluxDefId`] directly.
    pub fn require_builtin_assoc_reft(self, def_id: DefId, name: Symbol) -> FluxDefId {
        self.builtin_assoc_refts(def_id)
            .and_then(|assoc_refts| {
                assoc_refts
                    .items
                    .iter()
                    .find(|reft| reft.name() == name)
                    .map(|reft| reft.def_id())
            })
            .unwrap_or_else(|| bug!("missing builtin assoc reft `{name}` for `{def_id:?}`"))
    }

    /// Returns the body of a builtin associated refinement, or `None` if it cannot be computed
    /// yet, e.g., `size_of` for a type like `W<T>` whose layout depends on a generic parameter.
    /// In that case the caller should leave the alias unnormalized.
    pub fn builtin_assoc_reft_body(
        self,
        typing_env: rustc_middle::ty::TypingEnv<'tcx>,
        alias_reft: &AliasReft,
    ) -> Option<rty::Lambda> {
        let tcx = self.tcx();

        if tcx.is_lang_item(alias_reft.assoc_id.parent(), LangItem::Sized) {
            let self_ty = alias_reft.to_rustc_trait_ref(tcx).self_ty();
            // A type can be `Sized` via a builtin impl and still have a layout that depends on
            // generic parameters (`W<T>`, `[T; 2]`, ...), so `layout_of` may fail with
            // `TooGeneric`. Leaving the alias abstract is always sound.
            let layout = tcx.layout_of(typing_env.as_query_input(self_ty)).ok()?;
            let body = match alias_reft.assoc_id.name() {
                sym::size_of => rty::Expr::constant(rty::Constant::from(layout.size.bytes())),
                sym::align_of => rty::Expr::constant(rty::Constant::from(layout.align.abi.bytes())),
                _ => bug!("invalid builtin assoc reft {:?}", alias_reft.assoc_id),
            };
            Some(rty::Lambda::bind_with_vars(body, List::empty(), rty::Sort::Int))
        } else if tcx.is_lang_item(alias_reft.assoc_id.parent(), LangItem::FnOnce)
            && alias_reft.assoc_id.name() == sym::no_panic
        {
            let self_ty = alias_reft.self_ty();
            let body = match self_ty.as_bty_skipping_binder() {
                BaseTy::Closure(_, _, _, no_panic) => {
                    if *no_panic {
                        rty::Expr::tt()
                    } else {
                        rty::Expr::ff()
                    }
                }
                _ => rty::Expr::ff(),
            };
            Some(rty::Lambda::bind_with_vars(body, List::empty(), rty::Sort::Bool))
        } else {
            bug!("invalid builtin assoc reft {:?}", alias_reft.assoc_id)
        }
    }

    /// Whether `assoc_id` is one of the builtin associated refinements of `MetaSized`.
    pub fn is_meta_sized_assoc_reft(self, assoc_id: FluxDefId) -> bool {
        self.tcx()
            .is_lang_item(assoc_id.parent(), LangItem::MetaSized)
    }

    /// The body of `MetaSized::size_of_val` / `MetaSized::align_of_val` for the self type of
    /// `alias_reft`, or `None` if the alias cannot be normalized and must stay uninterpreted
    /// (e.g., `dyn Trait`, or a generic `T: ?Sized`).
    ///
    /// All impls of `MetaSized` are builtin, so unlike other associated refinements we don't go
    /// through trait selection. This lets us normalize the alias for a type parameter `T` with a
    /// `T: Sized` bound to `T::size_of()`, which selection would leave as a param-env candidate.
    ///
    /// The single argument is the pointer metadata: unused for sized types, the length for slices
    /// and `str`, and the vtable pointer for `dyn Trait`.
    pub fn meta_sized_assoc_reft_body(
        self,
        typing_env: TypingEnv<'tcx>,
        alias_reft: &AliasReft,
    ) -> Option<rty::Lambda> {
        let tcx = self.tcx();
        let name = alias_reft.assoc_id.name();
        let is_size = match name {
            sym::size_of_val => true,
            sym::align_of_val => false,
            _ => bug!("invalid builtin assoc reft {:?}", alias_reft.assoc_id),
        };
        let self_ty = alias_reft.to_rustc_trait_ref(tcx).self_ty();
        let meta = Expr::nu();

        let body = if self_ty.is_sized(tcx, typing_env) {
            // The metadata is `()`, so the argument is ignored.
            if let Ok(layout) = tcx.layout_of(typing_env.as_query_input(self_ty)) {
                let n = if is_size { layout.size.bytes() } else { layout.align.abi.bytes() };
                Expr::constant(rty::Constant::from(n))
            } else {
                let ctor = meta_sized_self_bty(&alias_reft.args[0])?.to_subset_ty_ctor();
                let name = if is_size { sym::size_of } else { sym::align_of };
                self.sized_alias(name, ctor)
            }
        } else {
            match meta_sized_self_bty(&alias_reft.args[0]) {
                Some(BaseTy::Slice(elem_ty)) => {
                    let ctor = elem_ty.as_bty_skipping_existentials()?.to_subset_ty_ctor();
                    if is_size {
                        Expr::binary_op(
                            BinOp::Mul(Sort::Int),
                            meta,
                            self.sized_alias(sym::size_of, ctor),
                        )
                    } else {
                        self.sized_alias(sym::align_of, ctor)
                    }
                }
                Some(BaseTy::Str) => {
                    if is_size {
                        meta
                    } else {
                        Expr::constant(rty::Constant::from(1u64))
                    }
                }
                // Structs and tuples with an unsized tail, `dyn Trait`, generic `T: ?Sized`, ...
                _ => {
                    if is_size {
                        concrete_dst_size(tcx, typing_env, self_ty, &meta)?
                    } else {
                        Expr::constant(rty::Constant::from(concrete_dst_align(
                            tcx, typing_env, self_ty,
                        )?))
                    }
                }
            }
        };
        Some(rty::Lambda::bind_with_fsort(body, rty::FuncSort::new(vec![Sort::Int], Sort::Int)))
    }

    /// `<ctor as Sized>::name()`
    fn sized_alias(self, name: Symbol, ctor: rty::SubsetTyCtor) -> Expr {
        let sized_id = self.tcx().require_lang_item(LangItem::Sized, DUMMY_SP);
        Expr::alias(
            AliasReft {
                assoc_id: self.require_builtin_assoc_reft(sized_id, name),
                args: List::from_arr([GenericArg::Base(ctor)]),
            },
            List::empty(),
        )
    }
}

/// The base type of the self argument of a `MetaSized` alias. `MetaSized` refines its self
/// parameter as a `Type` (see `generics_of`), but we also accept a `Base` argument.
fn meta_sized_self_bty(arg: &GenericArg) -> Option<&BaseTy> {
    match arg {
        GenericArg::Ty(ty) => ty.as_bty_skipping_existentials(),
        GenericArg::Base(ctor) => Some(ctor.as_bty_skipping_binder()),
        _ => None,
    }
}

/// The size of an unsized type whose layout is fully known, as a function of its metadata `meta`.
/// This mirrors `size_and_align_of_dst` in rustc's codegen: a struct (or tuple) with an unsized
/// tail has size `round_up(offset_of_tail + size_of_val(tail), align)`. Returns `None` if some part
/// of the type is not concrete or the tail is a `dyn Trait`.
fn concrete_dst_size<'tcx>(
    tcx: rustc_ty::TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    ty: rustc_ty::Ty<'tcx>,
    meta: &Expr,
) -> Option<Expr> {
    match ty.kind() {
        rustc_ty::Slice(elem) => {
            let elem_size = tcx
                .layout_of(typing_env.as_query_input(*elem))
                .ok()?
                .size
                .bytes();
            Some(Expr::binary_op(
                BinOp::Mul(Sort::Int),
                meta.clone(),
                Expr::constant(rty::Constant::from(elem_size)),
            ))
        }
        rustc_ty::Str => Some(meta.clone()),
        rustc_ty::Adt(..) | rustc_ty::Tuple(..) => {
            let (tail_ty, tail_offset) = unsized_tail(tcx, typing_env, ty)?;
            let align = concrete_dst_align(tcx, typing_env, ty)?;
            let unrounded = Expr::binary_op(
                BinOp::Add(Sort::Int),
                Expr::constant(rty::Constant::from(tail_offset)),
                concrete_dst_size(tcx, typing_env, tail_ty, meta)?,
            );
            Some(round_up(unrounded, align))
        }
        _ => None,
    }
}

/// The alignment of an unsized type whose layout is fully known. See [`concrete_dst_size`].
fn concrete_dst_align<'tcx>(
    tcx: rustc_ty::TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    ty: rustc_ty::Ty<'tcx>,
) -> Option<u64> {
    let align = match ty.kind() {
        rustc_ty::Slice(elem) => {
            tcx.layout_of(typing_env.as_query_input(*elem))
                .ok()?
                .align
                .abi
                .bytes()
        }
        rustc_ty::Str => 1,
        rustc_ty::Adt(..) | rustc_ty::Tuple(..) => {
            let (tail_ty, _) = unsized_tail(tcx, typing_env, ty)?;
            let layout = tcx.layout_of(typing_env.as_query_input(ty)).ok()?;
            // The alignment of the sized prefix combined with the alignment of the tail.
            let tail_align = concrete_dst_align(tcx, typing_env, tail_ty)?;
            u64::max(layout.align.abi.bytes(), tail_align)
        }
        _ => return None,
    };
    Some(align)
}

/// The last field of a struct or tuple, which must be the unsized one, together with its offset.
fn unsized_tail<'tcx>(
    tcx: rustc_ty::TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    ty: rustc_ty::Ty<'tcx>,
) -> Option<(rustc_ty::Ty<'tcx>, u64)> {
    let tail_ty = match ty.kind() {
        rustc_ty::Adt(adt_def, args) if adt_def.is_struct() => {
            let field_ty = adt_def
                .non_enum_variant()
                .fields
                .iter()
                .last()?
                .ty(tcx, args);
            tcx.try_normalize_erasing_regions(typing_env, field_ty)
                .ok()?
        }
        rustc_ty::Tuple(tys) => *tys.last()?,
        _ => return None,
    };
    let layout = tcx.layout_of(typing_env.as_query_input(ty)).ok()?;
    let last = layout.fields.count().checked_sub(1)?;
    Some((tail_ty, layout.fields.offset(last).bytes()))
}

/// `round_up(e, align)`, where `align` is a constant.
fn round_up(e: Expr, align: u64) -> Expr {
    if align == 1 {
        return e;
    }
    let align = Expr::constant(rty::Constant::from(align));
    let one = Expr::constant(rty::Constant::from(1u64));
    let bumped = Expr::binary_op(
        BinOp::Sub(Sort::Int),
        Expr::binary_op(BinOp::Add(Sort::Int), e, align.clone()),
        one,
    );
    Expr::binary_op(
        BinOp::Mul(Sort::Int),
        Expr::binary_op(BinOp::Div(Sort::Int), bumped, align.clone()),
        align,
    )
}
