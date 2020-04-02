use quote::quote;
use syn::{parse2, Field, Fields, ItemStruct, Path, Type, TypePath};

use crate::args::Args;
use crate::error::unexpected;

mod attrs;

const OPTION: &str = "Option";

/// Wraps item fields in Option.
pub fn generate(item: &ItemStruct, args: &Args) -> Fields {
    let item_name = item.ident.clone();

    let mut fields = item.fields.clone();

    for field in fields.iter_mut() {
        field.attrs = attrs::generate(field, args);
        attrs::generate(field, args);

        if is_option(&field) && !args.rewrap {
            continue;
        }

        let ty = field.ty.clone();

        let opt_type = quote! {
            Option<#ty>
        };

        field.ty = parse2(opt_type)
            .unwrap_or_else(|e| panic!(unexpected(format!("generating {} fields", item_name), e)));
    }

    fields
}

pub fn is_option(field: &Field) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::*;

    #[test]
    fn test_is_not_option() {
        let field = parse_field(quote! {
            field: String
        });

        assert!(!is_option(&field));
    }

    #[test]
    fn test_is_option() {
        let field = parse_field(quote! {
            field: Option<String>
        });

        assert!(is_option(&field));
    }
}
