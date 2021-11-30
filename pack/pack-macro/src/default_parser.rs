use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::DataStruct;

use crate::util::str_to_toks;

#[derive(Debug)]
pub struct FieldInfo {
    name: String,
    ty: String,
}

#[derive(Debug)]
pub struct DefaultParser {
    name: String,
    fields: Vec<FieldInfo>,
}

impl DefaultParser {
    pub fn parse(st: &DataStruct, name: String) -> Self {
        let mut instance = Self {
            name,
            fields: Vec::new(),
        };

        instance.parse_struct(st);

        instance
    }

    pub fn gen(&self) -> TokenStream {
        let name = str_to_toks(&self.name);
        let toks: Vec<TokenStream> = self
            .fields
            .iter()
            .map(|field| {
                let name = str_to_toks(&field.name);
                let ty = str_to_toks(&field.ty);
                quote! {
                    #name: <#ty>::pack_default(),
                }
            })
            .collect();
        quote! {
            impl PackDefault for #name {
                fn pack_default() -> Self {
                    Self {
                        #(#toks)*
                    }
                }
            }
        }
    }

    fn parse_struct(&mut self, st: &DataStruct) {
        for field in &st.fields {
            if let Some(ident) = &field.ident {
                let name = ident.to_string();
                let ty = field.ty.to_token_stream().to_string();

                self.fields.push(FieldInfo { ty, name });
            } else {
                abort!(field, "Field must have name")
            }
        }
    }
}
