#[macro_use]
mod util;
mod parser;
mod union_parser;

use parser::Parser;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use syn::{parse_macro_input, DeriveInput};
use union_parser::UnionParser;

#[proc_macro_derive(Pack, attributes(packed, len, value, pack_type))]
#[proc_macro_error]
pub fn derive_pack(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);

    let parser = Parser::parse(input);
    let stream = parser.gen();

    stream.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn pack_union(_: TokenStream, item: TokenStream) -> TokenStream {
    let item: syn::Item = parse_macro_input!(item);
    let uni = match item {
        syn::Item::Union(uni) => uni,
        i @ _ => abort! { i,
            "#[pack_union] only used for union block"
        },
    };

    let parser = UnionParser::parse(uni);
    let stream = parser.gen();

    stream.into()
}
