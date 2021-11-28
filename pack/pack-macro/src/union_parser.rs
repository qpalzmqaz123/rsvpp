use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::util::str_to_toks;

#[derive(Debug)]
pub struct FieldInfo {
    name: String,
    ty: String,
}

#[derive(Debug)]
pub struct UnionParser {
    name: String,
    attrs: Vec<String>,
    fields: Vec<FieldInfo>,
}

impl UnionParser {
    pub fn parse(uni: syn::ItemUnion) -> Self {
        let mut instance = Self {
            name: uni.ident.to_string(),
            attrs: Vec::new(),
            fields: Vec::new(),
        };
        instance.parse_union(uni);

        instance
    }

    pub fn gen(&self) -> TokenStream {
        let def_struct = self.gen_def_struct();
        let impl_pack = self.gen_pack_trait();
        let impl_self = self.gen_impl_self();

        quote! {
            #def_struct

            #impl_pack

            #impl_self
        }
    }

    fn parse_union(&mut self, uni: syn::ItemUnion) {
        self.attrs = uni
            .attrs
            .iter()
            .map(|attr| attr.to_token_stream().to_string())
            .collect::<Vec<String>>();

        self.fields = uni
            .fields
            .named
            .iter()
            .map(|field| {
                let name = field.ident.to_token_stream().to_string();
                let ty = field.ty.to_token_stream().to_string();

                FieldInfo { name, ty }
            })
            .collect::<Vec<FieldInfo>>();
    }

    fn gen_def_struct(&self) -> TokenStream {
        let name = str_to_toks(&self.name);
        let attr_toks = self
            .attrs
            .iter()
            .map(|attr| str_to_toks(attr))
            .collect::<Vec<TokenStream>>();

        quote! {
            #(#attr_toks)*
            pub struct #name {
                buf: Vec<u8>,
            }
        }
    }

    fn gen_pack_trait(&self) -> TokenStream {
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

    fn gen_size_fn_body(&self) -> TokenStream {
        quote! {
            Self::static_size()
        }
    }

    fn gen_static_size_fn_body(&self) -> TokenStream {
        let toks: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| {
                let ty = str_to_toks(&field.ty);
                quote! {
                    <#ty>::static_size()
                }
            })
            .collect();

        quote! {
            pack::max!(#(#toks),*)
        }
    }

    fn gen_align_size_fn_body(&self) -> TokenStream {
        let toks: Vec<TokenStream> = self
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
            pack::max!(#(#toks),*)
        }
    }

    fn gen_pack_fn_body(&self) -> TokenStream {
        quote! {
            if buf.len() < self.buf.len() {
                return Err("Buffer not enough".into());
            }

            self.buf.iter().enumerate().for_each(|(i, v)| buf[i] = *v);

            Ok(self.buf.len())
        }
    }

    fn gen_unpack_fn_body(&self) -> TokenStream {
        quote! {
            let len = Self::static_size();
            if buf.len() < len {
                return Err("Buffer not enough".into());
            }

            Ok((
                Self {
                    buf: (&buf[0..len]).to_vec(),
                },
                len,
            ))
        }
    }

    fn gen_impl_self(&self) -> TokenStream {
        let name = str_to_toks(&self.name);
        let get_toks = self
            .fields
            .iter()
            .map(|field| {
                let name = str_to_toks(&field.name);
                let ty = str_to_toks(&field.ty);
                quote! {
                    pub fn #name(&self) -> #ty {
                        <#ty>::unpack(&self.buf, 0).expect("Unpack error").0
                    }
                }
            })
            .collect::<Vec<TokenStream>>();
        let set_toks = self
            .fields
            .iter()
            .map(|field| {
                let name = str_to_toks(&format!("from_{}", field.name));
                let ty = str_to_toks(&field.ty);
                quote! {
                    pub fn #name(mut value: #ty) -> Self {
                        let mut buf = vec![0u8; Self::static_size()];
                        // TODO: Handle error
                        value.pack(&mut buf).expect("Pack error");

                        Self { buf }
                    }
                }
            })
            .collect::<Vec<TokenStream>>();

        quote! {
            impl #name {
                #(#get_toks)*

                #(#set_toks)*
            }
        }
    }
}
