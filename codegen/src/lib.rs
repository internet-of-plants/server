#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, Expr, Stmt, FnArg, token::Brace, Block, Item, punctuated::Pair, punctuated::Punctuated, token::Comma, Pat, Visibility};
use std::iter::FromIterator;

#[proc_macro_attribute]
pub fn exec_time(attr: TokenStream, item: TokenStream) -> TokenStream {
    assert!(attr.is_empty(), "#[exec_time] doesn't support attributes");

    let mut input = parse_macro_input!(item as ItemFn);
    let mut outer_function = input.clone();
    input.vis = Visibility::Inherited;
    let function = input.sig.ident.clone();
    let arguments: Punctuated<Pat, Comma> = Punctuated::from_iter(input.sig.inputs.pairs().map(|arg| {
        match arg {
            Pair::Punctuated(t, p) => match t {
                FnArg::Receiver(r) => unimplemented!("{:?}", r),
                FnArg::Typed(pat) => Pair::Punctuated((*pat.pat).clone(), *p),
            }
            Pair::End(t) => match t {
                FnArg::Receiver(r) => unimplemented!("{:?}", r),
                FnArg::Typed(pat) => Pair::End((*pat.pat).clone()),
            }
        }
    }));

    let (stmt, now)  = if input.sig.asyncness.is_some() {
        (Stmt::Expr(Expr::Verbatim(quote!{ let ret = #function(#arguments).await; })), quote! { ::tokio::time::Instant::now() })
    } else {
        (Stmt::Expr(Expr::Verbatim(quote!{ let ret = #function(#arguments); })), quote! { ::std::time::Instant::now() })
    };

    outer_function.attrs = vec![];
    outer_function.block = Box::new(Block {
        brace_token: Brace::default(),
        stmts: vec![
            Stmt::Item(Item::Fn(input)),
            Stmt::Expr(Expr::Verbatim(quote!{ let ___codegen_TIMER = #now; })),
            stmt,
            Stmt::Expr(Expr::Verbatim(quote!{ let ___codegen_TIMER2 = #now; })),
            Stmt::Expr(Expr::Verbatim(quote!{ 
                let time = (___codegen_TIMER2 - ___codegen_TIMER).as_micros();
                if time > 1_000_000 {
                    info!(target: "timer", "{} took {:.3}s to run ({})", stringify!(#function), (time as f64) / 1_000_000., file!());
                } else {
                    debug!(target: "timer", "{} took {:.3}s to run ({})", stringify!(#function), (time as f64) / 1_000_000., file!());
                }
            })),
            Stmt::Expr(Expr::Verbatim(quote! { ret }))
        ]
    });

    let ts: proc_macro2::TokenStream = quote! { 
        #outer_function
    };
    //println!("{}", ts);
    ts.into()
}

#[proc_macro_attribute]
pub fn cache(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attr as syn::AttributeArgs);

    let mut valid_for = u64::max_value();
    let mut max_size = 100;

    // Oh no
    for attribute in attributes {
        match attribute {
            syn::NestedMeta::Meta(meta) => match meta {
                syn::Meta::NameValue(word) => {
                    match word.path.to_token_stream().to_string().as_str() {
                        "valid_for" => match word.lit {
                            syn::Lit::Int(int) => valid_for = int.base10_parse::<u64>().unwrap(),
                            _ => unimplemented!("{:?}", word),
                        }
                        "max_size" => match word.lit {
                            syn::Lit::Int(int) => max_size = int.base10_parse::<usize>().unwrap(),
                            _ => unimplemented!("{:?}", word),
                        }
                        _ => unimplemented!("{:?}", word),
                    }
                },
                _ => panic!("Unsupported parameter type, try #[controller(inline, log)]"),
            },
            _ => panic!("Literals are not supported as attributes of #[controller]"),
        }
    }

    let mut input = parse_macro_input!(item as ItemFn);
    let return_type = match &input.sig.output {
        syn::ReturnType::Default => syn::Type::Verbatim(quote! { () }),
        syn::ReturnType::Type(_, ty) => match &**ty {
            syn::Type::Path(ty) => match &ty.path.segments.first().unwrap().arguments {
                syn::PathArguments::AngleBracketed(args) => match args.args.iter().next().unwrap() {
                    syn::GenericArgument::Type(ty) => ty.clone(),
                    _ => unimplemented!("{:?}", args),
                }
                _ => unimplemented!("{:?}", ty),
            }
            _ => unimplemented!("{:?}", ty),
        }
    };
    //println!("{:?}", return_type);

    let mut outer_function = input.clone();
    input.vis = Visibility::Inherited;
    let function = input.sig.ident.clone();
    let full_arguments: Punctuated<Pat, Comma> = Punctuated::from_iter(input.sig.inputs.pairs().flat_map(|arg| {
        match arg {
            Pair::Punctuated(t, p) => match t {
                FnArg::Receiver(r) => unimplemented!("{:?}", r),
                FnArg::Typed(pat) => Some(Pair::Punctuated((*pat.pat).clone(), *p))
            }
            Pair::End(t) => match t {
                FnArg::Receiver(r) => unimplemented!("{:?}", r),
                FnArg::Typed(pat) => Some(Pair::End((*pat.pat).clone()))
            }
        }
    }));

    let arguments: Punctuated<Pat, Comma> = Punctuated::from_iter(input.sig.inputs.pairs().flat_map(|arg| {
        match arg {
            Pair::Punctuated(t, p) => match t {
                FnArg::Receiver(r) => unimplemented!("{:?}", r),
                FnArg::Typed(pat) => {
                    if pat.ty.to_token_stream().to_string().contains("Pool") {
                        return None;
                    }
                    let pat = &*pat.pat;
                    Some(Pair::Punctuated(Pat::Verbatim(quote! { #pat.clone() }), *p))
                }
            }
            Pair::End(t) => match t {
                FnArg::Receiver(r) => unimplemented!("{:?}", r),
                FnArg::Typed(pat) => {
                    if pat.ty.to_token_stream().to_string().contains("Pool") {
                        return None;
                    }
                    let pat = &*pat.pat;
                    Some(Pair::End(Pat::Verbatim(quote! { #pat.clone() })))
                }
            }
        }
    }));
    let types: Punctuated<syn::Type, Comma> = Punctuated::from_iter(input.sig.inputs.pairs().flat_map(|arg| {
        match arg {
            Pair::Punctuated(t, p) => match t {
                FnArg::Receiver(r) => unimplemented!("{:?}", r),
                FnArg::Typed(pat) => {
                    if pat.ty.to_token_stream().to_string().contains("Pool") {
                        return None;
                    }
                    Some(Pair::Punctuated((*pat.ty).clone(), *p))
                }
            }
            Pair::End(t) => match t {
                FnArg::Receiver(r) => unimplemented!("{:?}", r),
                FnArg::Typed(pat) => {
                    if pat.ty.to_token_stream().to_string().contains("Pool") {
                        return None;
                    }
                    Some(Pair::End((*pat.ty).clone()))
                }
            }
        }
    }));

    let (stmt, instant) = if input.sig.asyncness.is_some() {
        (Stmt::Expr(Expr::Verbatim(quote!{ let ret = #function(#full_arguments).await; })), quote! { ::tokio::time::Instant })
    } else {
        (Stmt::Expr(Expr::Verbatim(quote!{ let ret = #function(#full_arguments); })), quote! { ::std::time::Instant })
    };

    outer_function.attrs = vec![];
    outer_function.block = Box::new(Block {
        brace_token: Brace::default(),
        stmts: vec![
            Stmt::Item(Item::Fn(input)),
            Stmt::Expr(Expr::Verbatim(quote!{ 
                thread_local! {
                    static CACHE: ::std::cell::RefCell<::std::collections::HashMap<(#types), (#return_type, #instant)>> = Default::default();
                }
                
                let key = (#arguments);
                let _now = #instant::now();
                if let Some((return_, last_update)) = CACHE.with(|cache| cache.borrow().get(&key).map(|r| r.clone())) {
                    let should_run = _now.saturating_duration_since(last_update).as_secs() > #valid_for;
                    //println!("Active {}, should run: {}", stringify!(#function), should_run);
                    if !should_run {
                        return Ok(return_);
                    }
                }
                // Whoever tries while we are running gets the stale value without ever realizing
                // shh bby is ok
                CACHE.with(|cache| {
                    if let Some((_, instant)) = cache.borrow_mut().get_mut(&key) {
                        *instant = _now;
                    }
                });
            })),
            stmt,
            Stmt::Expr(Expr::Verbatim(quote!{ 
                match &ret {
                    Ok(val) => CACHE.with(|cache| {
                        let mut cache = cache.borrow_mut();
                        cache.insert(key, (val.clone(), _now));

                        if cache.len() > #max_size {
                            let mut purge = vec![];
                            for (key, (_, instant)) in &*cache {
                                if _now.saturating_duration_since(*instant).as_secs() > #valid_for {
                                    purge.push(key.clone());
                                }
                            }

                            for key in purge {
                                cache.remove(&key);
                            }

                            if cache.len() > #max_size {
                                let mut remove = cache.len() - #max_size;
                                let mut pairs = Vec::with_capacity(remove);
                                for (key, (_, instant)) in &*cache {
                                    pairs.push((key.clone(), *instant));
                                }
                                pairs.sort_unstable_by_key(|(_, i)| *i);

                                for (key, _) in pairs {
                                    cache.remove(&key);

                                    if cache.len() < #max_size / 2 {
                                        break;
                                    }
                                }
                            }
                        }
                    }),
                    Err(err) => CACHE.with(|cache| { let _ = cache.borrow_mut().remove(&key); })
                };
                ret
            })),
        ]
    });

    let ts: proc_macro2::TokenStream = quote! { 
        #outer_function
    };
    //println!("{}", ts);
    ts.into()
}
