use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Error};

#[proc_macro_attribute]
pub fn memoize(_input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(annotated_item as syn::ItemFn);
    let attrs = input.attrs.clone();
    let vis = input.vis.clone();
    let sig = input.sig.clone();
    ensure_first_param_is_ui(&sig);
    let func_name = sig.ident.clone();

    let inputs = sig.inputs.clone();

    let memoize_arg = match inputs.len() {
        // no params except `ui`
        1 => {
            quote! {
                ()
            }
        }
        // 1 param except `ui`
        2 => {
            if let syn::FnArg::Typed(arg) = &inputs[1] {
                let pat = arg.clone().pat;
                quote! {
                    (#pat,)
                }
            } else {
                Error::new(func_name.span(), "Invalid param").to_compile_error()
            }
        }
        // more than 1 params except `ui`
        _ => {
            let mut args = Vec::new();
            for arg in inputs.iter().skip(1) {
                if let syn::FnArg::Typed(arg) = arg {
                    let pat = arg.clone().pat;
                    args.push(quote! {
                        #pat
                    });
                } else {
                    Error::new(func_name.span(), "Invalid param").to_compile_error();
                }
            }
            quote! {
                (#(#args),*)
            }
        }
    };

    // Build the output, possibly using quasi-quotation
    let new_function = quote! {
        #(#attrs)*
        #vis #sig {
            #input
            ui.memoize(#func_name, #memoize_arg);
        }
    };
    // Hand the output tokens back to the compiler
    TokenStream::from(new_function)
}

fn ensure_first_param_is_ui(sig: &syn::Signature) {
    if sig.inputs.len() < 1 {
        Error::new(sig.ident.span(), "expected at least one parameter").to_compile_error();
    }
    if let syn::FnArg::Typed(arg) = &sig.inputs[0] {
        if let syn::Pat::Ident(ident) = &*arg.pat {
            if ident.ident != "ui" {
                Error::new(
                    ident.ident.span(),
                    "memoize expects the first argument to be `ui`",
                )
                .to_compile_error();
            }
        }
    }
}
