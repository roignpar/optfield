use quote::quote;
use syn::{parse2, Field, Fields, Ident, ItemStruct, Path, Type, TypePath};

use crate::args::Args;
use crate::error::unexpected;

mod attrs;

/// Wraps item fields in Option.
pub fn generate(item: &ItemStruct, args: &Args) -> Fields {
    let item_name = item.ident.clone();

    let mut fields = item.fields.clone();

    let wrapper = args.final_wrapper();

    for field in fields.iter_mut() {
        field.attrs = attrs::generate(field, args);
        attrs::generate(field, args);

        if is_wrapped_in(field, &wrapper) && !args.rewrap {
            continue;
        }

        let ty = &field.ty;

        let wrapped_type = quote! {
            #wrapper<#ty>
        };

        field.ty = parse2(wrapped_type).unwrap_or_else(|e| {
            panic!(
                "{}",
                unexpected(format!("generating {} fields", item_name), e)
            )
        });
    }

    fields
}

pub fn is_wrapped_in(field: &Field, wrapper: &Ident) -> bool {
    match &field.ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => {
            if let Some(segment) = segments.first() {
                &segment.ident == wrapper
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

    use quote::format_ident;

    #[test]
    fn test_is_not_option() {
        let field = parse_field(quote! {
            field: String
        });

        assert!(!is_wrapped_in(&field, &format_ident!("Option")));
    }

    #[test]
    fn test_is_option() {
        let field = parse_field(quote! {
            field: Option<String>
        });

        assert!(is_wrapped_in(&field, &format_ident!("Option")));
    }

    #[test]
    fn test_is_not_wrapper() {
        let field = parse_field(quote! {
            field: String
        });

        assert!(!is_wrapped_in(&field, &format_ident!("Wrapper")));
    }

    #[test]
    fn test_is_wrapper() {
        let field = parse_field(quote! {
            field: Wrapper<String>
        });

        assert!(is_wrapped_in(&field, &format_ident!("Wrapper")));
    }

    #[test]
    fn without_rewrap() {
        let (item, args) = parse_item_and_args(
            quote! {
                struct S<T> {
                    string: Option<String>,
                    int: i32,
                    generic: T,
                    optional_generic: Option<T>
                }
            },
            quote! {
                Opt
            },
        );

        let expected_types = parse_types(vec![
            quote! {Option<String>},
            quote! {Option<i32>},
            quote! {Option<T>},
            quote! {Option<T>},
        ]);

        let generated = generate(&item, &args);

        assert_eq!(field_types(generated), expected_types);
    }

    #[test]
    fn with_rewrap() {
        let (item, args) = parse_item_and_args(
            quote! {
                struct S<T> {
                    text: String,
                    number: Option<i128>,
                    generic: T,
                    optional_generic: Option<T>
                }
            },
            quote! {
                Opt,
                rewrap
            },
        );

        let expected_types = parse_types(vec![
            quote! {Option<String>},
            quote! {Option<Option<i128>>},
            quote! {Option<T>},
            quote! {Option<Option<T>>},
        ]);

        let generated = generate(&item, &args);

        assert_eq!(field_types(generated), expected_types);
    }

    #[test]
    fn without_wrapper_rewrap() {
        let (item, args) = parse_item_and_args(
            quote! {
                struct S<T> {
                    optional_string: Option<String>,
                    int: i32,
                    generic: T,
                    wrapped_generic: Wrapper<T>
                }
            },
            quote! {
                Opt,
                wrapper = Wrapper
            },
        );

        let expected_types = parse_types(vec![
            quote! {Wrapper<Option<String>>},
            quote! {Wrapper<i32>},
            quote! {Wrapper<T>},
            quote! {Wrapper<T>},
        ]);

        let generated = generate(&item, &args);

        assert_eq!(field_types(generated), expected_types);
    }

    #[test]
    fn with_wrapper_rewrap() {
        let (item, args) = parse_item_and_args(
            quote! {
                struct S<T> {
                    text: String,
                    optional_number: Option<i128>,
                    generic: T,
                    wrapped_generic: Wrapper<T>
                }
            },
            quote! {
                Opt,
                wrapper = Wrapper,
                rewrap
            },
        );

        let expected_types = parse_types(vec![
            quote! {Wrapper<String>},
            quote! {Wrapper<Option<i128>>},
            quote! {Wrapper<T>},
            quote! {Wrapper<Wrapper<T>>},
        ]);

        let generated = generate(&item, &args);

        assert_eq!(field_types(generated), expected_types);
    }
}
