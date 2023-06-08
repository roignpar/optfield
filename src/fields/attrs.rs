use quote::quote;
use syn::{parse2, Attribute, Field, Meta};

use crate::args::{Args, Attrs, Doc};
use crate::attrs::generator::{is_doc_attr, AttrGenerator};
use crate::error::unexpected;
use crate::fields::args::FieldArgs;

const OPTFIELD_FIELD_ATTR_NAME: &str = "optfield";

struct FieldAttrGen<'a> {
    field: &'a Field,
    args: &'a Args,
}

impl<'a> FieldAttrGen<'a> {
    fn new(field: &'a Field, args: &'a Args) -> Self {
        Self { field, args }
    }

    /// Get the #[optfield(...)] args on this field, if the attribute exists.
    fn optfield_args(&self) -> Option<FieldArgs> {
        let mut optfield_field_attrs_it = self
            .field
            .attrs
            .iter()
            .filter_map(|attr| is_optfield_field_attr(attr).then(|| &attr.meta));

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

    fn new_attrs_except_docs(&self) -> Vec<Meta> {
        use Attrs::*;

        let struct_attrs_arg = &self.args.field_attrs;

        let original_attrs_it = self
            .field
            .attrs
            .iter()
            .filter_map(|attr| (!is_doc_attr(attr)).then(|| attr.meta.clone()));

        let attr_attrs_arg = self.optfield_args().and_then(|args| args.attrs);

        // field arg overrides struct arg
        match (struct_attrs_arg, attr_attrs_arg) {
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

    fn new_docs(&self) -> Vec<Meta> {
        use Doc::*;

        let struct_doc_arg = self.args.field_doc;

        let original_doc_it = self
            .field
            .attrs
            .iter()
            .filter_map(|attr| is_doc_attr(attr).then(|| attr.meta.clone()));

        let attr_doc_arg = self.optfield_args().and_then(|args| args.doc);

        // field arg overrides struct arg
        match (struct_doc_arg, attr_doc_arg) {
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
    use super::*;

    use quote::quote;

    use crate::test_util::*;

    #[test]
    fn remove_docs() {
        let field = parse_field(quote! {
            /// some
            /// doc
            /// lines
            #[attr]
            field: i32
        });

        let cases = vec![
            quote! {
                Opt

            },
            quote! {
                Opt,
                field_attrs
            },
            quote! {
                Opt,
                field_attrs = (new_attr)
            },
            quote! {
                Opt,
                field_attrs = add(new_attr)
            },
        ];

        let item_docs = doc_attrs(&field.attrs);

        for case in cases {
            let args = parse_args(case);

            let generated = generate(&field, &args);

            assert!(!attrs_contain_any(&generated, &item_docs));
        }
    }

    #[test]
    fn keep_docs() {
        let field = parse_field(quote! {
            /// field
            /// with
            /// docs
            #[attr]
            field: String
        });

        let cases = vec![
            quote! {
                Opt,
                field_doc
            },
            quote! {
                Opt,
                field_doc,
                field_attrs
            },
            quote! {
                Opt,
                field_doc,
                field_attrs = (new_attr)
            },
            quote! {
                Opt,
                field_doc,
                field_attrs = add(new_attr)
            },
        ];

        let item_docs = doc_attrs(&field.attrs);

        for case in cases {
            let args = parse_args(case);

            let generated = generate(&field, &args);

            assert!(attrs_contain_all(&generated, &item_docs));
        }
    }

    #[test]
    fn remove_attrs() {
        let (field, args) = parse_field_and_args(
            quote! {
                #[some]
                #[attrs]
                field: String
            },
            quote! {
                Opt
            },
        );

        let generated = generate(&field, &args);

        assert!(!attrs_contain_any(&generated, &field.attrs));
    }

    #[test]
    fn keep_attrs() {
        let (field, args) = parse_field_and_args(
            quote! {
                #[some]
                #[attrs]
                field: String
            },
            quote! {
                Opt,
                field_attrs
            },
        );

        let generated = generate(&field, &args);

        assert!(attrs_contain_all(&generated, &field.attrs));
    }

    #[test]
    fn replace_attrs() {
        let (field, args) = parse_field_and_args(
            quote! {
                #[some]
                #[attrs]
                field: String
            },
            quote! {
                Opt,
                field_attrs = (
                    other,
                    new,
                    attribute
                )
            },
        );

        let new_attrs = parse_attrs(quote! {
            #[other]
            #[new]
            #[attribute]
        });

        let generated = generate(&field, &args);

        assert!(!attrs_contain_any(&generated, &field.attrs));
        assert!(attrs_contain_all(&generated, &new_attrs));
    }

    #[test]
    fn add_attrs() {
        let (field, args) = parse_field_and_args(
            quote! {
                #[old]
                #[attrs]
                field: u8
            },
            quote! {
                Opt,
                field_attrs = add(
                    new,
                    field,
                    attributes
                )
            },
        );

        let new_attrs = parse_attrs(quote! {
            #[new]
            #[field]
            #[attributes]
        });

        let generated = generate(&field, &args);

        assert!(attrs_contain_all(&generated, &field.attrs));
        assert!(attrs_contain_all(&generated, &new_attrs));
    }
}
