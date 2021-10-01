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

    pub args_struct: (),
    // /// Attributes and signature information.
    // pub attr_sig_info: AttrSigInfo,
    // /// String representation of method name, e.g. `"my_method"`.
    // pub ident_byte_str: LitStr,
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

        let generic_lifetimes = original
            .sig
            .generics
            .lifetimes()
            .map(|lt| (lt.lifetime.ident.clone(), lt.clone()))
            .collect();
        let generic_types =
            original.sig.generics.type_params().map(|tp| (tp.ident.clone(), tp.clone())).collect();
        let generic_consts =
            original.sig.generics.const_params().map(|cp| (cp.ident.clone(), cp.clone())).collect();

        let mut receiver = None;
        let mut args = indexmap::IndexMap::new();
        for ref arg in original.sig.inputs {
            match arg {
                syn::FnArg::Receiver(r) => {
                    assert!(receiver.is_none());
                    receiver = Some(r.clone())
                }
                syn::FnArg::Typed(pty) => match *pty.pat {
                    syn::Pat::Ident(pt) => {
                        args.insert(pt.ident.clone(), pty.clone());
                    }
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
                    _ => {
                        return Err(Error::new(
                            pty.span(),
                            "Only identity patterns, such as `x: bool`, are supported in method arguments",
                        ));
                    }
                },
            }
        }

        // let TraitItemMethod { attrs, sig, .. } = original;

        // TODO: continue from here
        for (_ident, pat_type) in args {
            match *pat_type.ty {
                // [T; n]
                syn::Type::Array(ta) => todo!(),
                //  fn(usize) -> bool
                syn::Type::BareFn(_) => todo!(),
                //
                syn::Type::Group(_) => todo!(),
                // impl Bound1 + Bound2 + Bound3
                syn::Type::ImplTrait(_) => todo!(),
                // _
                syn::Type::Infer(_) => todo!(),
                //
                syn::Type::Macro(_) => todo!(),
                // !
                syn::Type::Never(_) => todo!(),
                //
                syn::Type::Paren(_) => todo!(),
                // std::slice::Iter, <Vec<T> as SomeTrait>::Associated
                syn::Type::Path(_) => todo!(),
                // *const T
                syn::Type::Ptr(_) => todo!(),
                // &'a mut T
                syn::Type::Reference(_) => todo!(),
                // [T]
                syn::Type::Slice(_) => todo!(),
                // Bound1 + Bound2 + Bound3
                syn::Type::TraitObject(_) => todo!(),
                // (A, B, C, String)
                syn::Type::Tuple(_) => todo!(),
                // ..
                syn::Type::Verbatim(_) => todo!(),
                _ => todo!(),
            }
        }

        let args_struct = {};

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
            args_struct,
            // attr_sig_info,
            // ident_byte_str
        })
    }
}
