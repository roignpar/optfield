use quote::quote;
use syn::{parse2, Field, ItemStruct, Path, Type, TypePath};

use crate::args::Args;
use crate::error::unexpected;

mod attrs;

const OPTION: &str = "Option";

/// Wraps item fields in Option.
pub fn generate(item: &mut ItemStruct, args: &Args) {
    let item_name = item.ident.clone();

    for field in item.fields.iter_mut() {
        attrs::generate(field, args);

        if field_is_option(&field) && !args.rewrap {
            continue;
        }

        let ty = field.ty.clone();

        let opt_type = quote! {
            Option<#ty>
        };

        field.ty = parse2(opt_type)
            .unwrap_or_else(|e| panic!(unexpected(format!("generating {} fields", item_name), e)));
    }
}

fn field_is_option(field: &Field) -> bool {
    match &field.ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => {
            if let Some(segment) = segments.first() {
                segment.ident == OPTION
            } else {
                false
            }
        }
        _ => false,
    }
}
