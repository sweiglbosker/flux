use std::sync::OnceLock;

use flux_arc_interner::List;
use flux_common::bug;
use flux_rustc_bridge::lowering::Lower as _;
use flux_syntax::symbols::sym;
use rustc_attr_ir::LangItem;
use rustc_data_structures::unord::UnordMap;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{self as rustc_ty, TypingEnv};
use rustc_span::{DUMMY_SP, Symbol};

use crate::{
    def_id::FluxDefId,
    global_env::GlobalEnv,
    queries::QueryResult,
    query_bug,
    rty::{
        self, AliasReft, AssocRefinements, AssocReft, BaseTy, BinOp, Expr, Sort, refining::Refiner,
    },
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

    /// The body of a builtin associated refinement for a builtin impl, or `None` if it cannot be
    /// computed and the alias must stay uninterpreted.
    ///
    /// For `MetaSized`, this is also called when the impl comes from the param env, because a type
    /// parameter `T: Sized` has `T: MetaSized` as an elaborated bound. In that case, the alias is
    /// rewritten to the corresponding `Sized` alias.
    pub fn builtin_assoc_reft_body(
        self,
        typing_env: TypingEnv<'tcx>,
        refiner: &Refiner<'_, 'tcx>,
        alias_reft: &AliasReft,
    ) -> QueryResult<Option<rty::Lambda>> {
        let tcx = self.tcx();
        let trait_id = alias_reft.assoc_id.parent();
        let name = alias_reft.assoc_id.name();

        let lambda = if tcx.is_lang_item(trait_id, LangItem::Sized) {
            let self_ty = alias_reft.to_rustc_trait_ref(tcx).self_ty();
            // A type can be `Sized` via a builtin impl and still have a layout that depends on
            // generic parameters (`W<T>`, `[T; 2]`, ...), so `layout_of` may fail with
            // `TooGeneric`. Leaving the alias abstract is always sound.
            let Ok(layout) = tcx.layout_of(typing_env.as_query_input(self_ty)) else {
                return Ok(None);
            };
            let body = match name {
                sym::size_of => Expr::constant(rty::Constant::from(layout.size.bytes())),
                sym::align_of => Expr::constant(rty::Constant::from(layout.align.abi.bytes())),
                _ => bug!("invalid builtin assoc reft {:?}", alias_reft.assoc_id),
            };
            rty::Lambda::bind_with_vars(body, List::empty(), Sort::Int)
        } else if tcx.is_lang_item(trait_id, LangItem::MetaSized) {
            let self_ty = alias_reft.to_rustc_trait_ref(tcx).self_ty();
            let Some(body) = self.meta_sized_body(typing_env, refiner, name, self_ty)? else {
                return Ok(None);
            };
            rty::Lambda::bind_with_fsort(body, rty::FuncSort::new(vec![Sort::Int], Sort::Int))
        } else if tcx.is_lang_item(trait_id, LangItem::FnOnce) && name == sym::no_panic {
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
            rty::Lambda::bind_with_vars(body, List::empty(), Sort::Bool)
        } else {
            bug!("invalid builtin assoc reft {:?}", alias_reft.assoc_id)
        };
        Ok(Some(lambda))
    }

    /// Whether `assoc_id` is one of the builtin associated refinements of `MetaSized`.
    pub fn is_meta_sized_assoc_reft(self, assoc_id: FluxDefId) -> bool {
        self.tcx()
            .is_lang_item(assoc_id.parent(), LangItem::MetaSized)
    }

    /// The body of `MetaSized::size_of_val` / `MetaSized::align_of_val` for `self_ty`, in terms of
    /// the pointer metadata `meta` (bound variable 0): unused for sized types, the length for slices
    /// and `str`, and the vtable pointer for `dyn Trait`.
    fn meta_sized_body(
        self,
        typing_env: TypingEnv<'tcx>,
        refiner: &Refiner<'_, 'tcx>,
        name: Symbol,
        self_ty: rustc_ty::Ty<'tcx>,
    ) -> QueryResult<Option<Expr>> {
        let tcx = self.tcx();
        let is_size = match name {
            sym::size_of_val => true,
            sym::align_of_val => false,
            _ => bug!("invalid builtin assoc reft `{name}` for `MetaSized`"),
        };
        let sized = |ty| self.builtin_alias(refiner, LangItem::Sized, ty, List::empty());
        let meta = Expr::nu();

        // Sized types ignore the metadata
        if self_ty.is_sized(tcx, typing_env) {
            let name = if is_size { sym::size_of } else { sym::align_of };
            return Ok(Some(sized(self_ty)?(name)));
        }

        let body = match self_ty.kind() {
            rustc_ty::Slice(elem) if is_size => {
                Expr::binary_op(BinOp::Mul(Sort::Int), meta, sized(*elem)?(sym::size_of))
            }
            rustc_ty::Slice(elem) => sized(*elem)?(sym::align_of),
            rustc_ty::Str if is_size => meta,
            rustc_ty::Str => Expr::constant(rty::Constant::from(1u64)),
            // A struct (or tuple) whose last field is unsized has size
            // `round_up(offset_of_tail + size_of_val(tail), align)`. The offset of a `dyn Trait`
            // tail depends on its dynamic alignment, so we only handle slice and `str` tails, for
            // which the layout of the struct is static.
            rustc_ty::Adt(..) | rustc_ty::Tuple(..) => {
                let tail = tcx.struct_tail_for_codegen(self_ty, typing_env);
                if !matches!(tail.kind(), rustc_ty::Slice(_) | rustc_ty::Str) {
                    return Ok(None);
                }
                let Ok(layout) = tcx.layout_of(typing_env.as_query_input(self_ty)) else {
                    return Ok(None);
                };
                let align = layout.align.abi.bytes();
                if !is_size {
                    return Ok(Some(Expr::constant(rty::Constant::from(align))));
                }
                let Some(last_field) = last_field_ty(tcx, typing_env, self_ty) else {
                    return Ok(None);
                };
                let offset = layout.fields.offset(layout.fields.count() - 1).bytes();
                let tail_size = self.builtin_alias(
                    refiner,
                    LangItem::MetaSized,
                    last_field,
                    List::singleton(meta),
                )?(sym::size_of_val);
                round_up(
                    Expr::binary_op(
                        BinOp::Add(Sort::Int),
                        Expr::constant(rty::Constant::from(offset)),
                        tail_size,
                    ),
                    align,
                )
            }
            // `dyn Trait`, a type parameter `T: ?Sized`, ...
            _ => return Ok(None),
        };
        Ok(Some(body))
    }

    /// Returns a function building `<ty as Trait>::name(args)` for the builtin `Trait`.
    fn builtin_alias(
        self,
        refiner: &Refiner<'_, 'tcx>,
        lang_item: LangItem,
        ty: rustc_ty::Ty<'tcx>,
        refine_args: List<Expr>,
    ) -> QueryResult<impl Fn(Symbol) -> Expr> {
        let tcx = self.tcx();
        let trait_id = tcx.require_lang_item(lang_item, DUMMY_SP);
        let args = tcx
            .mk_args(&[ty.into()])
            .lower(tcx)
            .map_err(|reason| query_bug!(trait_id, "{reason:?}"))?;
        let args = refiner.refine_generic_args(trait_id, &args)?;
        Ok(move |name| {
            let assoc_id = self.require_builtin_assoc_reft(trait_id, name);
            Expr::alias(AliasReft { assoc_id, args: args.clone() }, refine_args.clone())
        })
    }
}

/// The type of the last field of a struct or tuple.
fn last_field_ty<'tcx>(
    tcx: rustc_ty::TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    ty: rustc_ty::Ty<'tcx>,
) -> Option<rustc_ty::Ty<'tcx>> {
    match ty.kind() {
        rustc_ty::Adt(adt_def, args) if adt_def.is_struct() => {
            let field_ty = adt_def
                .non_enum_variant()
                .fields
                .iter()
                .last()?
                .ty(tcx, args);
            tcx.try_normalize_erasing_regions(typing_env, field_ty).ok()
        }
        rustc_ty::Tuple(tys) => tys.last().copied(),
        _ => None,
    }
}

/// `round_up(e, align)` for a constant `align`.
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
