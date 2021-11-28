mod enum_parser;
mod struct_parser;

use enum_parser::EnumParser;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use struct_parser::StructParser;
use syn::{Data, DeriveInput};

use crate::util::parse_string_arg;

#[derive(Debug)]
pub enum Parser {
    Struct(StructParser),
    Enum(EnumParser),
}

impl Parser {
    pub fn parse(input: DeriveInput) -> Self {
        let name = input.ident.to_string();
        let mut packed = false;
        let mut pack_type = "u32".to_string();

        for attr in &input.attrs {
            if let Some(seg) = attr.path.segments.first() {
                match seg.ident.to_string().as_str() {
                    "packed" => packed = true,
                    "pack_type" => match parse_string_arg(&attr.tokens) {
                        Ok(s) => pack_type = s,
                        Err(_) => abort!(attr, "Syntax error"),
                    },
                    _ => abort!(attr, "Syntax error"),
                }
            }
        }

        match &input.data {
            Data::Struct(st) => Self::Struct(StructParser::parse(st, name, packed)),
            Data::Enum(en) => Self::Enum(EnumParser::parse(en, name, pack_type)),
            _ => abort!(input, "Unsupport"),
        }
    }

    pub fn gen(&self) -> TokenStream {
        match &self {
            Self::Struct(st) => st.gen(),
            Self::Enum(en) => en.gen(),
        }
    }
}
