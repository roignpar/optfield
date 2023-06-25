use quote::quote;
use syn::{parse2, Attribute, Field, Meta};

use crate::args::{Args, Attrs, Doc};
use crate::attrs::generator::{is_doc_attr, AttrGenerator};
use crate::error::unexpected;
use crate::fields::args::FieldArgs;

/// Without helper attribute support on `proc_macro_attribute`, this attribute
/// name needs to be different from the name of the proc macro (`optfield`) to
/// avoid namespace conflicts.
const OPTFIELD_FIELD_ATTR_NAME: &str = "optfield_field";

#[derive(Debug)]
struct FieldAttrGen<'a> {
    field: &'a Field,
    args: &'a Args,
}

impl<'a> FieldAttrGen<'a> {
    fn new(field: &'a Field, args: &'a Args) -> Self {
        Self { field, args }
    }

    /// Get the #[optfield_field(...)] args on this field, if the attribute exists.
    fn optfield_field_args(&self) -> Option<FieldArgs> {
        let mut optfield_field_attrs_it = self.field.attrs.iter().filter_map(|attr| {
            if is_optfield_field_attr(attr) {
                Some(&attr.meta)
            } else {
                None
            }
        });

        let attr = match optfield_field_attrs_it.next() {
            Some(attr) => attr,
            None => return None,
        };

        if optfield_field_attrs_it.next().is_some() {
            panic!("There can be at most 1 optfield attribute on each field.");
        }

        let args: FieldArgs = match attr {
            Meta::Path(_) | Meta::NameValue(_) => panic!("Expected parentheses."),
            Meta::List(list) => list.parse_args().unwrap_or_else(|e| panic!("{}", e)),
        };

        Some(args)
    }
}

impl<'a> AttrGenerator for FieldAttrGen<'a> {
    fn error_action_text(&self) -> String {
        let field_name = match &self.field.ident {
            None => "".to_string(),
            Some(ident) => format!("{} ", ident),
        };

        format!("generating {}field attrs", field_name)
    }

    fn new_non_doc_attrs(&self) -> Vec<Meta> {
        use Attrs::*;

        let struct_attrs_arg = &self.args.field_attrs;

        let original_attrs_it = self
            .field
            .attrs
            .iter()
            // #[optfield(..)] attributes do not need to be regenerated
            .filter(|attr| !is_optfield_field_attr(attr))
            .filter_map(|attr| (!is_doc_attr(attr)).then(|| attr.meta.clone()));

        let field_attrs_arg = self.optfield_field_args().and_then(|args| args.attrs);

        // field arg overrides struct arg
        match (struct_attrs_arg, field_attrs_arg) {
            // no attributes
            (None, None) => vec![],
            // keep original
            (Some(Keep), None) => original_attrs_it.collect(),
            // replace with attributes defined on struct
            (Some(Replace(attrs)), None) => attrs.clone(),
            // add attributes defined on struct
            (Some(Add(attrs)), None) => original_attrs_it.chain(attrs.iter().cloned()).collect(),
            // keep original (field arg override)
            (_, Some(Keep)) => original_attrs_it.collect(),
            // replace with attributes defined on field (field arg override)
            (_, Some(Replace(attrs))) => attrs,
            // add attributes defined on field (field arg override)
            (_, Some(Add(attrs))) => original_attrs_it.chain(attrs).collect(),
        }
    }

    fn new_doc_attrs(&self) -> Vec<Meta> {
        use Doc::*;

        let struct_doc_arg = self.args.field_doc;

        let original_doc_it = self
            .field
            .attrs
            .iter()
            .filter_map(|attr| is_doc_attr(attr).then(|| attr.meta.clone()));

        let field_doc_arg = self.optfield_field_args().and_then(|args| args.doc);

        // field arg overrides struct arg
        match (struct_doc_arg, field_doc_arg) {
            // no docs
            (false, None) => vec![],
            // keep original
            (true, None) => original_doc_it.collect(),
            // keep original (field arg override)
            (_, Some(Keep)) => original_doc_it.collect(),
            // replace with doc on field
            (_, Some(Replace(doc_text))) => {
                let tokens = quote! { doc = #doc_text };
                let doc_attr = parse2(tokens)
                    .unwrap_or_else(|e| panic!("{}", unexpected(self.error_action_text(), e)));
                vec![doc_attr]
            }
            // append to original with doc on field
            (_, Some(Append(doc_text))) => {
                let tokens = quote! { doc = #doc_text };
                let doc_attr = parse2(tokens)
                    .unwrap_or_else(|e| panic!("{}", unexpected(self.error_action_text(), e)));
                original_doc_it.chain(std::iter::once(doc_attr)).collect()
            }
        }
    }
}

pub fn generate(field: &Field, args: &Args) -> Vec<Attribute> {
    FieldAttrGen::new(field, args).generate()
}

/// This currently has the same implementation as `is_optfield_attr`, but they
/// are semantically distinct, and therefore have separate functions.
pub fn is_optfield_field_attr(attr: &Attribute) -> bool {
    attr.path().is_ident(OPTFIELD_FIELD_ATTR_NAME)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    use proc_macro2::{Ident, Span, TokenStream};
    use quote::quote;
    use rstest::{fixture, rstest};

    use crate::test_util::*;

    #[fixture]
    fn field() -> Field {
        parse_field(quote! {
            /// some
            /// doc
            /// lines
            #[attr]
            field: i32
        })
    }

    /// Shared impl for expected results.
    trait ExpectedAttrs {
        fn into_expected_attrs(self) -> HashSet<Attribute>;
        /// Chain multiple possibly-indeterminate expected results.
        ///
        /// Similar to [`Option::or`].
        fn or(self, other: Self) -> Self;
    }

    /// The possible expected generated attributes for [`test_field_doc_attrs_only`].
    #[derive(Debug, Copy, Clone)]
    enum ExpectedDocAttrs {
        Unknown,
        Discarded,
        Kept,
        Replaced,
        Appended,
    }
    impl ExpectedAttrs for ExpectedDocAttrs {
        fn into_expected_attrs(self) -> HashSet<Attribute> {
            use ExpectedDocAttrs::*;

            // we are only testing for doc attributes
            let base_it = field().attrs.into_iter().filter(|a| is_doc_attr(a));

            match self {
                Unknown => unreachable!("The expected doc attributes depend on other factors"),
                Discarded => HashSet::new(),
                Kept => base_it.collect(),
                Replaced => std::iter::once(parse_attr(quote!(#[doc = "replaced"]))).collect(),
                Appended => base_it
                    .chain(std::iter::once(parse_attr(quote!(#[doc = "appended"]))))
                    .collect(),
            }
        }
        fn or(self, other: Self) -> Self {
            use ExpectedDocAttrs::*;
            match self {
                Unknown => other,
                Discarded | Kept | Replaced | Appended => self,
            }
        }
    }

    /// The possible expected generated attributes for [`test_field_non_doc_attrs_only`].
    #[derive(Debug, Copy, Clone)]
    enum ExpectedNonDocAttrs {
        Unknown,
        Discarded,
        KeptByStruct,
        KeptByField,
        ReplacedByStruct,
        ReplacedByField,
        AddedByStruct,
        AddedByField,
    }
    impl ExpectedAttrs for ExpectedNonDocAttrs {
        fn into_expected_attrs(self) -> HashSet<Attribute> {
            use ExpectedNonDocAttrs::*;

            // we are only testing for non-doc attributes
            let base_it = field().attrs.into_iter().filter(|a| !is_doc_attr(a));

            match self {
                Unknown => unreachable!("The expected non-doc attributes depend on other factors"),
                Discarded => HashSet::new(),
                KeptByStruct | KeptByField => base_it.collect(),
                ReplacedByStruct => parse_attrs(quote! {
                    #[replaced_by_struct_0]
                    #[replaced_by_struct_1]
                })
                .into_iter()
                .collect(),
                ReplacedByField => parse_attrs(quote! {
                    #[replaced_by_field_0]
                    #[replaced_by_field_1]
                })
                .into_iter()
                .collect(),
                AddedByStruct => base_it
                    .chain(parse_attrs(quote! {
                        #[added_by_struct_0]
                        #[added_by_struct_1]
                    }))
                    .collect(),
                AddedByField => base_it
                    .chain(parse_attrs(quote! {
                        #[added_by_field_0]
                        #[added_by_field_1]
                    }))
                    .collect(),
            }
        }
        fn or(self, other: Self) -> Self {
            use ExpectedNonDocAttrs::*;
            match self {
                Unknown => other,
                Discarded | KeptByStruct | KeptByField | ReplacedByStruct | ReplacedByField
                | AddedByStruct | AddedByField => self,
            }
        }
    }

    fn test_field_attrs<T>(
        mut field: Field,
        (struct_args, struct_expected): (TokenStream, T),
        (field_args, field_expected): (TokenStream, T),
    ) where
        T: ExpectedAttrs,
    {
        // setup: insert field args into base field
        let optfield_field_attr_ident = Ident::new(OPTFIELD_FIELD_ATTR_NAME, Span::call_site());
        let optfield_field_attr = parse_attr(quote!(#[#optfield_field_attr_ident(#field_args)]));
        field.attrs.push(optfield_field_attr);

        let generated: HashSet<_> = generate(&field, &parse_struct_args(struct_args))
            .into_iter()
            .collect();
        // field args override struct args
        let expected = field_expected.or(struct_expected).into_expected_attrs();
        dbg!(&generated, &expected);

        assert_eq!(generated, expected);
    }

    #[rstest]
    fn test_field_doc_attrs_only(
        field: Field,
        // .0 are the args on struct; .1 are the expected docs (unless overridden)
        #[values(
            (quote!(Opt), ExpectedDocAttrs::Discarded),
            (quote!(Opt, field_doc), ExpectedDocAttrs::Kept),
        )]
        struct_args_pair: (TokenStream, ExpectedDocAttrs),
        // .0 are the args on field; .1 are the expected docs (overriding)
        #[values(
            (quote!(), ExpectedDocAttrs::Unknown),
            (quote!(doc), ExpectedDocAttrs::Kept),
            (quote!(doc = "replaced"), ExpectedDocAttrs::Replaced),
            (quote!(doc = append("appended")), ExpectedDocAttrs::Appended),
        )]
        field_args_pair: (TokenStream, ExpectedDocAttrs),
    ) {
        test_field_attrs(field, struct_args_pair, field_args_pair)
    }

    #[rstest]
    fn test_field_non_doc_attrs_only(
        field: Field,
        // .0 are the args on struct; .1 are the expected attrs (unless overridden)
        #[values(
            (quote!(Opt), ExpectedNonDocAttrs::Discarded),
            (quote!(Opt, field_attrs), ExpectedNonDocAttrs::KeptByStruct),
            (quote!(Opt, field_attrs = (replaced_by_struct_0, replaced_by_struct_1)), ExpectedNonDocAttrs::ReplacedByStruct),
            (quote!(Opt, field_attrs = add(added_by_struct_0, added_by_struct_1)), ExpectedNonDocAttrs::AddedByStruct),
        )]
        struct_args_pair: (TokenStream, ExpectedNonDocAttrs),
        // .0 are the args on field; .1 are the expected attrs (overriding)
        #[values(
            (quote!(), ExpectedNonDocAttrs::Unknown),
            (quote!(attrs), ExpectedNonDocAttrs::KeptByField),
            (quote!(attrs = (replaced_by_field_0, replaced_by_field_1)), ExpectedNonDocAttrs::ReplacedByField),
            (quote!(attrs = add(added_by_field_0, added_by_field_1)), ExpectedNonDocAttrs::AddedByField),
        )]
        field_args_pair: (TokenStream, ExpectedNonDocAttrs),
    ) {
        test_field_attrs(field, struct_args_pair, field_args_pair)
    }
}
