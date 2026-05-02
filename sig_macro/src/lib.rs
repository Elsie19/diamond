use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemFn, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// Generate signature matching code.
///
/// # Example
/// If you have a function header like this:
///
/// ```text
/// let ~internal func(arr: [integer], string: string);
/// ```
///
/// You can use the proc macro like this:
///
/// ```
/// #[signature(args => arr: [integer], string: string)]
/// ```
///
/// Where `args` refers to the actual rust function's argument list.
#[proc_macro_attribute]
pub fn signature(attr: TokenStream, item: TokenStream) -> TokenStream {
    let sig = parse_macro_input!(attr as SignatureInput);
    let mut func = parse_macro_input!(item as ItemFn);

    let args_name = &sig.input_ident;
    let arg_len = sig.args.len();

    let destructure = sig.to_pattern();
    let array_checking = sig.array_checking();

    let stmt: syn::Stmt = match syn::parse2(destructure) {
        Ok(stmt) => stmt,
        Err(e) => return e.into_compile_error().into(),
    };
    func.block.stmts.insert(0, stmt);

    if !array_checking.is_empty() {
        let stmt: syn::Stmt = match syn::parse2(array_checking) {
            Ok(stmt) => stmt,
            Err(e) => return e.into_compile_error().into(),
        };
        func.block.stmts.insert(1, stmt);
    }

    func.block.stmts.insert(
        0,
        syn::parse_quote! {
            unsafe { ::std::hint::assert_unchecked(#args_name.len() == #arg_len); }
        },
    );

    quote!(#func).into()
}

struct SignatureInput {
    input_ident: Ident,
    args: Vec<Arg>,
}

struct Arg {
    name: Ident,
    ty: SigType,
}

enum SigType {
    Integer,
    String,
    Unit,
    Result,
    Array(Box<Self>),
    Stream,
    File,
    Any,
}

impl SigType {
    fn matches(&self, var: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match self {
            Self::Integer => quote! { matches!(#var, ILitType::Integer(_)) },
            Self::String => quote! { matches!(#var, ILitType::String(_)) },
            Self::Unit => quote! { matches!(#var, ILitType::Unit) },
            Self::Result => quote! { matches!(#var, ILitType::Result(_)) },
            Self::Stream => quote! { matches!(#var, ILitType::Stream(_)) },
            Self::File => quote! { matches!(#var, ILitType::File(_)) },
            Self::Any => quote! { true },
            Self::Array(sig_type) => {
                let inner_check = sig_type.matches(&quote! { v });
                quote! {
                    match #var {
                        ILitType::Array(arr) => arr.iter().all(|v| #inner_check),
                        _ => false,
                    }
                }
            }
        }
    }
}

impl Parse for SignatureInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let input_ident: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;

        let args = Punctuated::<Arg, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect();

        Ok(SignatureInput { input_ident, args })
    }
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let ty = input.parse()?;

        Ok(Arg { name, ty })
    }
}

impl Parse for SigType {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            let inner: SigType = content.parse()?;
            Ok(Self::Array(Box::new(inner)))
        } else {
            let ident: Ident = input.parse()?;

            match ident.to_string().as_str() {
                "integer" => Ok(Self::Integer),
                "string" => Ok(Self::String),
                "any" => Ok(Self::Any),
                "file" => Ok(Self::File),
                "unit" => Ok(Self::Unit),
                "stream" => Ok(Self::Stream),
                "result" => {
                    let content;
                    syn::parenthesized!(content in input);
                    let _ok_ty: SigType = content.parse()?;
                    content.parse::<Token![,]>()?;
                    let _err_ty: SigType = content.parse()?;

                    Ok(Self::Result)
                }
                _ => Err(syn::Error::new(
                    ident.span(),
                    format!("unknown type `{}`", ident),
                )),
            }
        }
    }
}

impl SignatureInput {
    fn to_pattern(&self) -> proc_macro2::TokenStream {
        let var_name = &self.input_ident;

        let patterns = self.args.iter().map(|arg| {
            let name = &arg.name;

            match &arg.ty {
                SigType::Array(_) => {
                    quote! { ILitType::Array(#name) }
                }
                SigType::Integer => {
                    quote! { ILitType::Integer(#name) }
                }
                SigType::String => {
                    quote! { ILitType::String(#name) }
                }
                SigType::File => {
                    quote! { ILitType::File(#name) }
                }
                SigType::Stream => {
                    quote! { ILitType::Stream(#name) }
                }
                SigType::Unit => {
                    quote! { ILitType::Unit }
                }
                SigType::Result => {
                    quote! { ILitType::Result(#name) }
                }
                SigType::Any => {
                    quote! { #name }
                }
            }
        });

        quote! {
            let [#(#patterns),*] = #var_name else {
                unsafe { ::std::hint::unreachable_unchecked() }
            };
        }
    }

    fn array_checking(&self) -> proc_macro2::TokenStream {
        let checks = self.args.iter().filter_map(|arg| {
            let name = &arg.name;

            if let SigType::Array(inner) = &arg.ty {
                let check = inner.matches(&quote! { v });

                Some(quote! {
                    if !#name.iter().all(|v| #check) {
                        unsafe { ::std::hint::unreachable_unchecked() }
                    }
                })
            } else {
                None
            }
        });

        quote! {
            #(#checks)*
        }
    }
}
