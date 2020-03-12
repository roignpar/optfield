extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

mod args;
mod docs;
mod error;
mod generate;

use args::Args;
use generate::generate;

#[proc_macro_attribute]
pub fn optfield(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: ItemStruct = parse_macro_input!(item);
    let args: Args = parse_macro_input!(attr);

    let opt_item = generate(args, &item);

    let out = quote! {
        #item

        #opt_item
    };

    out.into()
}
