use crate::util::str_to_toks;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens};
use syn::DataStruct;

#[derive(Debug)]
enum LengthInfo {
    Fixed(usize),
    Refer(String),
}

#[derive(Debug)]
struct FieldInfo {
    ty: String,
    name: String,
    length: Option<LengthInfo>,
}

#[derive(Debug)]
pub struct StructParser {
    name: String,
    packed: bool,
    fields: Vec<FieldInfo>,
}

impl StructParser {
    pub fn parse(st: &DataStruct, name: String, packed: bool) -> Self {
        let mut instance = Self {
            name,
            packed,
            fields: Vec::new(),
        };

        instance.parse_struct(st);

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

    fn parse_struct(&mut self, st: &DataStruct) {
        for field in &st.fields {
            if let Some(ident) = &field.ident {
                let name = ident.to_string();
                let ty = field.ty.to_token_stream().to_string();
                let mut length: Option<LengthInfo> = None;

                // Parse attr
                for attr in &field.attrs {
                    if let Some(seg) = attr.path.segments.first() {
                        match seg.ident.to_string().as_str() {
                            "len" => {
                                let expr: syn::ExprParen = parse2!(attr.tokens, "Syntax error");
                                if let syn::Expr::Lit(lit) = expr.expr.as_ref() {
                                    match &lit.lit {
                                        syn::Lit::Int(lit) => {
                                            if let Ok(n) = lit.base10_parse::<usize>() {
                                                length = Some(LengthInfo::Fixed(n));
                                            } else {
                                                abort!(lit, "Parse to number error");
                                            }
                                        }
                                        syn::Lit::Str(lit) => {
                                            length = Some(LengthInfo::Refer(lit.value()));
                                        }
                                        _ => abort!(attr, "Len must be int or str"),
                                    }
                                } else {
                                    abort!(attr, "Syntax error");
                                }
                            }
                            _ => abort!(attr, "Syntax error"),
                        }
                    }
                }

                self.fields.push(FieldInfo { ty, name, length });
            } else {
                abort!(field, "Field must have name")
            }
        }
    }

    fn gen_size_fn_body(&self) -> TokenStream {
        let packed = self.packed;
        let toks: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| {
                let name = str_to_toks(&field.name);
                let ty = str_to_toks(&field.ty);
                if let Some(LengthInfo::Fixed(len)) = field.length {
                    quote! {
                        offset = pack::align_offset(offset, <#ty>::align_size(), #packed);
                        offset += #len;
                    }
                } else {
                    quote! {
                        offset = pack::align_offset(offset, <#ty>::align_size(), #packed);
                        offset += self.#name.size();
                    }
                }
            })
            .collect();

        quote! {
            let mut offset = 0;

            #(#toks)*

            offset = pack::align_offset(offset, Self::align_size(), #packed);
            offset
        }
    }

    fn gen_static_size_fn_body(&self) -> TokenStream {
        let packed = self.packed;
        let toks: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| {
                let ty = str_to_toks(&field.ty);

                if field.ty == "String" {
                    if let Some(LengthInfo::Fixed(len)) = field.length {
                        // String with len is [u8; N]
                        return quote! {
                            offset = pack::align_offset(offset, <#ty>::align_size(), #packed);
                            offset += <[u8; #len]>::static_size();
                        };
                    }
                }

                quote! {
                    offset = pack::align_offset(offset, <#ty>::align_size(), #packed);
                    offset += <#ty>::static_size();
                }
            })
            .collect();

        quote! {
            let mut offset = 0;

            #(#toks)*

            offset = pack::align_offset(offset, Self::align_size(), #packed);
            offset
        }
    }

    fn gen_align_size_fn_body(&self) -> TokenStream {
        let aligns: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| {
                let ty = str_to_toks(&field.ty);
                quote! {
                    <#ty>::align_size()
                }
            })
            .collect();

        quote! {
            pack::max!(#(#aligns),*)
        }
    }

    fn gen_pack_fn_body(&self) -> TokenStream {
        let packed = self.packed;
        let rewrite_toks: Vec<TokenStream> = self.fields.iter().fold(Vec::new(), |mut v, f| {
            if let Some(LengthInfo::Refer(r)) = &f.length {
                let refer_ty = if let Some(info) = self.fields.iter().find(|info| &info.name == r) {
                    str_to_toks(&info.ty)
                } else {
                    panic!("Refer '{}' not found", r);
                };
                let refer = str_to_toks(&r);
                let name = str_to_toks(&f.name);
                v.push(quote! {
                    self.#refer = self.#name.len() as #refer_ty;
                });
            }
            v
        });

        let toks: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| {
                let name = str_to_toks(&field.name);
                let ty = str_to_toks(&field.ty);
                if let Some(LengthInfo::Fixed(len)) = field.length {
                    quote! {
                        offset = pack::align_offset(offset, <#ty>::align_size(), #packed);
                        self.#name.pack(pack::safe_slice_mut(buf, offset, Some(self.#name.size()))?)?;
                        offset += #len;
                    }
                } else {
                    quote! {
                        offset = pack::align_offset(offset, <#ty>::align_size(), #packed);
                        offset += self.#name.pack(pack::safe_slice_mut(buf, offset, Some(self.#name.size()))?)?;
                    }
                }
            })
            .collect();

        quote! {
            #(#rewrite_toks)*

            let mut offset = 0;

            #(#toks)*

            Ok(offset)
        }
    }

    fn gen_unpack_fn_body(&self) -> TokenStream {
        let packed = self.packed;
        let toks: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| {
                let name = format_ident!("__{}__", field.name);
                let ty = str_to_toks(&field.ty);

                let len = if let Some(LengthInfo::Refer(r)) = &field.length {
                    let refer = format_ident!("__{}__", r);
                    quote! { #refer as usize }
                } else {
                    quote! { 0 }
                };

                let size = if let Some(LengthInfo::Fixed(size)) = &field.length {
                    quote! { #size }
                } else {
                    quote! { res.1 }
                };

                quote! {
                    offset = pack::align_offset(offset, <#ty>::align_size(), #packed);
                    let res = <#ty>::unpack(pack::safe_slice(&buf, offset, None)?, #len)?;
                    let #name = res.0;
                    offset += #size;
                }
            })
            .collect();

        let fields: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|f| {
                let name = str_to_toks(&f.name);
                let id = format_ident!("__{}__", f.name);
                quote! { #name: #id }
            })
            .collect();

        quote! {
            let mut offset = 0;

            #(#toks)*

            Ok((Self {
                #(#fields),*
            }, offset))
        }
    }
}
