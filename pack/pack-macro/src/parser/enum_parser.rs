use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::DataEnum;

use crate::util::str_to_toks;

#[derive(Debug)]
struct FieldInfo {
    name: String,
    value: usize,
}

#[derive(Debug)]
pub struct EnumParser {
    name: String,
    ty: String,
    fields: Vec<FieldInfo>,
}

impl EnumParser {
    pub fn parse(en: &DataEnum, name: String, ty: String) -> Self {
        let mut instance = Self {
            name,
            ty,
            fields: Vec::new(),
        };

        instance.parse_enum(en);

        instance
    }

    pub fn gen(&self) -> TokenStream {
        let name = str_to_toks(&self.name);
        let size_fn_body = self.gen_size_fn_body();
        let static_size_fn_body = self.gen_static_size_fn_body();
        let align_size_fn_body = self.gen_align_size_fn_body();
        let pack_fn_body = self.gen_pack_fn_body();
        let unpack_fn_body = self.gen_unpack_fn_body();
        let tok = quote! {
            impl Pack for #name {
                fn size(&self) -> usize {
                    #size_fn_body
                }

                fn static_size() -> usize {
                    #static_size_fn_body
                }

                fn align_size() -> usize {
                    #align_size_fn_body
                }

                fn pack(&mut self, buf: &mut [u8]) -> pack::Result<usize> {
                    #pack_fn_body
                }

                fn unpack(buf: &[u8], _: usize) -> pack::Result<(Self, usize)> {
                    #unpack_fn_body
                }
            }
        };
        tok.into()
    }

    fn parse_enum(&mut self, en: &DataEnum) {
        for var in &en.variants {
            // Get first attr
            let attr = if let Some(at) = var.attrs.first() {
                at
            } else {
                abort!(var, "Expect attr")
            };

            // Ensure first attr is 'value'
            if let Some(seg) = attr.path.segments.first() {
                if seg.ident.to_string() != "value" {
                    abort!(var, "Attr must be 'value'")
                }
            } else {
                abort!(var, "Expect valid attr")
            };

            // Parse attr
            let expr: syn::ExprParen = parse2!(attr.tokens, "Syntax error");
            let value = if let syn::Expr::Lit(lit) = expr.expr.as_ref() {
                match &lit.lit {
                    syn::Lit::Int(lit) => {
                        if let Ok(n) = lit.base10_parse::<usize>() {
                            n
                        } else {
                            abort!(lit, "Parse to number error");
                        }
                    }
                    _ => abort!(attr, "Len must be int or str"),
                }
            } else {
                abort!(attr, "Syntax error");
            };

            // Insert field
            let name = var.ident.to_string();
            self.fields.push(FieldInfo { name, value });
        }
    }

    fn gen_size_fn_body(&self) -> TokenStream {
        quote! {
            Self::align_size()
        }
    }

    fn gen_static_size_fn_body(&self) -> TokenStream {
        quote! {
            Self::align_size()
        }
    }

    fn gen_align_size_fn_body(&self) -> TokenStream {
        let ty = str_to_toks(&self.ty);
        quote! {
            #ty::align_size()
        }
    }

    fn gen_pack_fn_body(&self) -> TokenStream {
        let toks: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| {
                let name = str_to_toks(&field.name);
                let ty = str_to_toks(&self.ty);
                let value = field.value;
                quote! {
                    Self::#name => (#value as #ty).pack(buf),
                }
            })
            .collect();

        quote! {
            match self {
                #(#toks)*
            }
        }
    }

    fn gen_unpack_fn_body(&self) -> TokenStream {
        let toks: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| {
                let value = str_to_toks(&field.value.to_string());
                let name = str_to_toks(&field.name);
                quote! {
                    #value => Self::#name,
                }
            })
            .collect();
        let ty = str_to_toks(&self.ty);

        quote! {
            let (v, size) = #ty::unpack(buf, 0)?;
            let e = match v {
                #(#toks)*
                _ => return Err(format!("Enum Pack received invalid number: '{}'", v).into()),
            };

            Ok((e, size))
        }
    }
}
