use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

use crate::args::Args;
use crate::{docs, fields};

pub fn generate(args: Args, original: &ItemStruct) -> TokenStream {
    let mut opt_struct = original.clone();

    opt_struct.ident = args.name.clone();
    opt_struct.vis = args.final_visibility();

    docs::generate(&mut opt_struct, &args);
    fields::generate(&mut opt_struct, &args);

    quote!(#opt_struct)
}
