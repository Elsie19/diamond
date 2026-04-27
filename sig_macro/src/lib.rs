use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemFn, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

#[proc_macro_attribute]
pub fn signature(attr: TokenStream, item: TokenStream) -> TokenStream {
    let sig = parse_macro_input!(attr as SignatureInput);
    let mut func = parse_macro_input!(item as ItemFn);

    let destructure = sig.to_pattern();

    func.block.stmts.insert(
        0,
        syn::parse_quote! {
            #destructure
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
    Array,
    Stream,
    File,
    Any,
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
            let _inner: SigType = content.parse()?;
            Ok(Self::Array)
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
                SigType::Array => {
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
                unreachable!("type checked");
            };
        }
    }
}
