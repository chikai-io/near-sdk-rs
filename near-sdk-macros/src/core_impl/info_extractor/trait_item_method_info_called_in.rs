use std::collections::HashMap;

use super::attr_sig_info_called_in::AttrSigInfo;
use syn::export::Span;
use syn::spanned::Spanned;
use syn::{Error, LitStr, TraitItemMethod};

/// Information extracted from trait method.
pub struct TraitItemMethodInfo {
    /// The original AST of the trait item method.
    pub original: TraitItemMethod,

    /// The method documentation.
    /// eg. `#[doc = "My Documentation"] fn f() {}`
    pub docs: Vec<syn::Lit>,

    /// The method lifetimes generics.  
    /// eg. `fn f<'a>(){}`.
    pub generic_lifetimes: indexmap::IndexMap<syn::Ident, syn::LifetimeDef>,
    /// The method type generics.  
    /// eg. `fn f<T>(){}`.
    pub generic_types: indexmap::IndexMap<syn::Ident, syn::TypeParam>,
    /// The trait const generics.  
    /// eg. `f f<const N: usize>(){}`
    pub generic_consts: indexmap::IndexMap<syn::Ident, syn::ConstParam>,

    /// The `self`, or `&mut self`, or `&self` part.
    pub receiver: Option<syn::Receiver>,

    pub args: indexmap::IndexMap<syn::Ident, syn::PatType>,

    pub args_sets: ArgsSets,
    //
    // /// Attributes and signature information.
    // pub attr_sig_info: AttrSigInfo,
    // /// String representation of method name, e.g. `"my_method"`.
    // pub ident_byte_str: LitStr,
}

/// Argument's requirements.
#[derive(Clone, Default)]
pub struct ArgsSets {
    /// eg. `trait Trait<'a>`.
    pub trait_generic_lifetimes: indexmap::IndexSet<syn::Ident>,
    /// eg. `trait Trait<T>`.
    pub trait_generic_types: indexmap::IndexSet<syn::Ident>,
    /// eg. `trait Trait<const N: usize>`
    pub trait_generic_consts: indexmap::IndexSet<syn::Ident>,
    /// eg. `trait Trait<'a>: 'a`.
    pub self_lifetime_bounds: indexmap::IndexSet<syn::Lifetime>,
    /// eg. `trait Trait: OtherTrait`.
    pub self_trait_bounds: indexmap::IndexSet<syn::TraitBound>,
    /// eg. `trait Trait<'a, T> where T: 'a`.
    pub trait_lifetime_bounds: indexmap::IndexSet<syn::Ident>,
    /// eg. `trait Trait<T> where T: Clone`.
    pub trait_type_bounds: indexmap::IndexSet<syn::Type>,
    /// eg. `trait Trait {const T: u8}`.
    pub trait_const_items: indexmap::IndexSet<syn::Ident>,
    /// eg. `trait Trait {type T}`.
    pub trait_assoc_type_items: indexmap::IndexSet<syn::Ident>,
    //
    /// eg. `fn f<'a>(){}`.
    pub method_generic_lifetimes: indexmap::IndexSet<syn::Ident>,
    /// eg. `fn f<T>(){}`.
    pub method_generic_types: indexmap::IndexSet<syn::Ident>,
    /// eg. `f f<const N: usize>(){}`
    pub method_generic_consts: indexmap::IndexSet<syn::Ident>,
}

impl TraitItemMethodInfo {
    pub fn new(
        original: &TraitItemMethod,
        trait_info: &super::item_trait_info_called_in::ItemTraitInfo,
    ) -> syn::Result<Self> {
        let mut docs = vec![];
        for attr in &original.attrs {
            if !matches!(attr.style, syn::AttrStyle::Outer) {
                continue;
            }

            if attr.path.is_ident("doc") {
                match attr.parse_meta()? {
                    syn::Meta::NameValue(mnv) => docs.push(mnv.lit),
                    bad => return Err(Error::new_spanned(bad, "unrecognized doc attribute")),
                };
            }
        }

        let generic_lifetimes: indexmap::IndexMap<syn::Ident, syn::LifetimeDef> = original
            .sig
            .generics
            .lifetimes()
            .map(|lt| (lt.lifetime.ident.clone(), lt.clone()))
            .collect();
        let generic_types: indexmap::IndexMap<syn::Ident, syn::TypeParam> =
            original.sig.generics.type_params().map(|tp| (tp.ident.clone(), tp.clone())).collect();
        let generic_consts: indexmap::IndexMap<syn::Ident, syn::ConstParam> =
            original.sig.generics.const_params().map(|cp| (cp.ident.clone(), cp.clone())).collect();

        let mut receiver = None;
        let mut args = indexmap::IndexMap::new();
        for arg in &original.sig.inputs {
            // TODO: consider supporting other kinds of arguments.
            // eg. `(a, b): (u8, bool)`.
            // (..and other much much more complicated cases..?)
            //
            // Note: a "single" argument may insert various
            // individual arguments (such as `a` and `b`).
            //
            // Note: this also affects the `interface::CalledIn`
            // implementation, since, for example in this argument
            // position, the function will be expecting a `(u8, bool)`
            // value, such as `(args.a, args.b)`.
            // this is not trivial to tract because there are cases
            // for complex tuples, for structs, etc.

            match arg {
                syn::FnArg::Receiver(r) => {
                    assert!(receiver.is_none());
                    receiver = Some(r.clone())
                }
                syn::FnArg::Typed(pty) => match pty.pat.as_ref() {
                    syn::Pat::Ident(pt) => {
                        args.insert(pt.ident.clone(), pty.clone());
                    }
                    syn::Pat::Box(pb) => {
                        return Err(Error::new(
                            pb.span(),
                            "support for box pattern `box T` is not implemented",
                        ));
                    }
                    syn::Pat::Lit(pl) => {
                        return Err(Error::new(
                            pl.span(),
                            "support for literal pattern is not implemented",
                        ));
                    }
                    syn::Pat::Macro(pm) => {
                        return Err(Error::new(
                            pm.span(),
                            "support for macro pattern is not implemented",
                        ));
                    }
                    syn::Pat::Or(po) => {
                        return Err(Error::new(
                            po.span(),
                            "support for Or pattern is not implemented",
                        ));
                    }
                    syn::Pat::Path(pp) => {
                        return Err(Error::new(
                            pp.span(),
                            "support for path pattern is not implemented",
                        ));
                    }
                    syn::Pat::Range(pr) => {
                        return Err(Error::new(
                            pr.span(),
                            "support for range pattern is not implemented",
                        ));
                    }
                    syn::Pat::Reference(pr) => {
                        return Err(Error::new(
                            pr.span(),
                            "support for reference pattern is not implemented",
                        ));
                    }
                    syn::Pat::Rest(pr) => {
                        return Err(Error::new(
                            pr.span(),
                            "support for Rest pattern is not implemented",
                        ));
                    }
                    syn::Pat::Slice(ps) => {
                        return Err(Error::new(
                            ps.span(),
                            "support for slice pattern is not implemented",
                        ));
                    }
                    syn::Pat::Struct(ps) => {
                        return Err(Error::new(
                            ps.span(),
                            "support for struct pattern is not implemented",
                        ));
                    }
                    syn::Pat::Tuple(pt) => {
                        return Err(Error::new(
                            pt.span(),
                            "support for tuple pattern is not implemented",
                        ));
                    }
                    syn::Pat::TupleStruct(pts) => {
                        return Err(Error::new(
                            pts.span(),
                            "support for tuple struct pattern is not implemented",
                        ));
                    }
                    syn::Pat::Type(pt) => {
                        return Err(Error::new(
                            pt.span(),
                            "support for type pattern is not implemented",
                        ));
                    }
                    syn::Pat::Verbatim(ts) => {
                        return Err(Error::new(
                            ts.span(),
                            "support for arbitrary token stream is not implemented",
                        ));
                    }
                    syn::Pat::Wild(pt) => {
                        return Err(Error::new(
                            pt.span(),
                            "support for wild-token pattern is not implemented",
                        ));
                    }
                    p => {
                        return Err(Error::new(
                            p.span(),
                            "support for unknown pattern is not implemented",
                        ));
                    }
                },
            }
        }

        // let TraitItemMethod { attrs, sig, .. } = original;

        use indexmap::IndexSet;

        let mut args_sets = ArgsSets::default();

        for (_ident, pat_type) in &args {
            match pat_type.ty.as_ref() {
                // [T; n]
                syn::Type::Array(ta) => {
                    return Err(Error::new(
                        ta.span(),
                        "support for arrays `[T; n]` is not implemented",
                    ));
                }
                //  fn(usize) -> bool
                syn::Type::BareFn(tbf) => {
                    return Err(Error::new(
                        tbf.span(),
                        "support for bare functions `fn() -> Type` is not implemented",
                    ));
                }
                //
                syn::Type::Group(tg) => {
                    return Err(Error::new(tg.span(), "support for Groups is not implemented"));
                }
                // impl Bound1 + Bound2 + Bound3
                syn::Type::ImplTrait(tit) => {
                    return Err(Error::new(
                        tit.span(),
                        "support for impl trait `impl Bound1 + Bound2` is not implemented",
                    ));
                }
                // _
                syn::Type::Infer(ti) => {
                    return Err(Error::new(
                        ti.span(),
                        "support for inferred type `_` is not implemented",
                    ));
                }
                //
                syn::Type::Macro(tm) => {
                    return Err(Error::new(
                        tm.span(),
                        "support for type macros `m!()` is not implemented",
                    ));
                }
                // !
                syn::Type::Never(tn) => {
                    return Err(Error::new(
                        tn.span(),
                        "support for never type `!` is not implemented",
                    ));
                }
                //
                syn::Type::Paren(tp) => {
                    return Err(Error::new(
                        tp.span(),
                        "support for parenthesys type is not implemented",
                    ));
                }
                // std::slice::Iter, <Vec<T> as SomeTrait>::Associated
                syn::Type::Path(p) => {
                    if let Some(ref _qself) = p.qself {
                        return Err(Error::new(
                            p.span(),
                            "support for <T as Trait>::Type is not implemented",
                        ));
                    }

                    if let Some(colons) = p.path.leading_colon {
                        return Err(Error::new(
                            colons.span(),
                            "support for ::Type is not implemented",
                        ));
                    }

                    if let Some(ident) = p.path.get_ident() {
                        if let Some(_tp) = trait_info.generic_types.get(ident) {
                            args_sets.trait_generic_types.insert(ident.clone());
                        } else if let Some(_tp) = generic_types.get(ident) {
                            args_sets.method_generic_types.insert(ident.clone());
                        } else {
                            // normal type, such as bool
                            // (do nothing)
                        }
                    } else {
                        return Err(Error::new(
                            p.path.span(),
                            "only support for single basic types is implemented",
                        ));
                    }
                }
                // *const T
                syn::Type::Ptr(tp) => {
                    return Err(Error::new(
                        tp.span(),
                        "support for pointer types `*const T` is not implemented",
                    ));
                }
                // &'a mut T
                syn::Type::Reference(tr) => {
                    return Err(Error::new(
                        tr.span(),
                        "support for reference types `&'lt mut T` is not implemented",
                    ));
                }
                // [T]
                syn::Type::Slice(ts) => {
                    return Err(Error::new(
                        ts.span(),
                        "support for slice types `[T]` is not implemented",
                    ));
                }
                // Bound1 + Bound2 + Bound3
                syn::Type::TraitObject(tto) => {
                    return Err(Error::new(
                        tto.span(),
                        "support for trait object types `Bound1 + Bound2` is not implemented",
                    ));
                }
                // (A, B, C, String)
                syn::Type::Tuple(tt) => {
                    return Err(Error::new(
                        tt.span(),
                        "support for tuple types `(T1, T2)` is not implemented",
                    ));
                }
                // ..
                syn::Type::Verbatim(ts) => {
                    return Err(Error::new(
                        ts.span(),
                        "support for arbitrary token stream is not implemented",
                    ));
                }
                t => {
                    return Err(Error::new(
                        t.span(),
                        "support for unkown types is not implemented",
                    ));
                }
            }
        }

        // let attr_sig_info = AttrSigInfo::new(attrs, sig)?;

        // let ident_byte_str = LitStr::new(&attr_sig_info.ident.to_string(), Span::call_site());

        Ok(Self {
            original: original.clone(),

            docs,
            generic_lifetimes,
            generic_types,
            generic_consts,
            receiver,
            args,
            args_sets,
            // attr_sig_info,
            // ident_byte_str
        })
    }
}
