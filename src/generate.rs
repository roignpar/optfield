use syn::ItemStruct;

use crate::args::Args;
use crate::docs;

pub fn generate(args: Args, original: &ItemStruct) -> ItemStruct {
    let mut opt_struct = original.clone();

    opt_struct.ident = args.name.clone();
    opt_struct.vis = args.final_visibility();

    docs::generate(&mut opt_struct, &args);

    opt_struct
}
