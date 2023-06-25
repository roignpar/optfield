use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, ItemStruct};

use crate::args::Args;
use crate::fields::attrs::is_optfield_field_attr;
use crate::{attrs, fields, from, merge};

pub fn generate(original: &mut ItemStruct, args: Args) -> TokenStream {
    let mut opt_struct = original.clone();

    opt_struct.ident = args.item.name.clone();
    opt_struct.vis = args.item.final_visibility();

    // the original struct def with at most one `optfield_field` attribute on each
    // field
    //
    // we do this here so that each `generate` sub-step need not worry about
    // having potentially multiple `optfield_field` attributes
    let original_sanitised = {
        let mut o = original.clone();
        for field in o.fields.iter_mut() {
            if let Some(idx) = index_of_first_optfield_field_attr(field) {
                let attrs_remaining: Vec<_> = field
                    .attrs
                    .drain(idx + 1..)
                    .filter(|attr| !is_optfield_field_attr(attr))
                    .collect();
                field.attrs.extend(attrs_remaining);
            }
        }
        o
    };

    opt_struct.attrs = attrs::generate(&original_sanitised, &args);
    opt_struct.fields = fields::generate(&original_sanitised, &args);

    let merge_impl = merge::generate(&original_sanitised, &opt_struct, &args);

    let from_impl = from::generate(&original_sanitised, &opt_struct, &args);

    // manually remove the first `optfield_field` attribute from each field of the
    // original struct
    //
    // we only remove the first because each `optfield` struct attribute only
    // consumes at most one `optfield_field` attribute on each field
    //
    // this manual handling may become unnecessary in the future
    // see https://github.com/rust-lang/rust/issues/65823
    for field in original.fields.iter_mut() {
        if let Some(idx) = index_of_first_optfield_field_attr(field) {
            field.attrs.remove(idx);
        }
    }

    quote! {
        #opt_struct

        #merge_impl

        #from_impl
    }
}

fn index_of_first_optfield_field_attr(field: &Field) -> Option<usize> {
    field
        .attrs
        .iter()
        .enumerate()
        .find_map(|(idx, attr)| is_optfield_field_attr(attr).then(|| idx))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::*;

    #[test]
    fn sets_name() {
        let (mut item, args) = parse_item_and_args(
            quote! {
                struct S;
            },
            quote! {
                Opt
            },
        );

        let generated = parse_item(generate(&mut item, args));

        assert_eq!(generated.ident, "Opt");
    }

    #[test]
    fn sets_generics() {
        let (mut item, args) = parse_item_and_args(
            quote! {
                struct S<'a, 'b, T, G> {
                    t: &'a T,
                    g: &'b G
                }
            },
            quote! {
                Opt
            },
        );

        let generated = parse_item(generate(&mut item, args));

        assert_eq!(item.generics, generated.generics);
    }

    #[test]
    fn sets_visibility() {
        let mut item = parse_item(quote! {
            struct S;
        });

        let cases = vec![
            (
                quote! {
                    Opt
                },
                quote!(),
            ),
            (
                quote! {
                    pub Opt
                },
                quote!(pub),
            ),
            (
                quote! {
                    pub(crate) Opt
                },
                quote!(pub(crate)),
            ),
            (
                quote! {
                    pub(in test::path) Opt
                },
                quote!(pub(in test::path)),
            ),
        ];

        for (args_tokens, vis_tokens) in cases {
            let args = parse_struct_args(args_tokens);
            let vis = parse_visibility(vis_tokens);

            let generated = parse_item(generate(&mut item, args));

            assert_eq!(generated.vis, vis);
        }
    }
}
