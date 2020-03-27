use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

use crate::args::Args;
use crate::{attrs, fields, merge};

pub fn generate(args: Args, original: &ItemStruct) -> TokenStream {
    let mut opt_struct = original.clone();

    opt_struct.ident = args.name.clone();
    opt_struct.vis = args.final_visibility();

    opt_struct.attrs = attrs::generate(original, &args);
    opt_struct.fields = fields::generate(original, &args);

    let merge_impl = merge::generate(original, &opt_struct, &args);

    quote! {
        #opt_struct

        #merge_impl
    }
}
