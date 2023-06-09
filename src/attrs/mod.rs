use quote::quote;
use syn::{parse2, Attribute, ItemStruct, Meta};

use crate::args::{Args, Attrs, Doc};
use crate::attrs::generator::{is_doc_attr, is_optfield_attr};
use crate::error::unexpected;

pub mod generator;

use generator::AttrGenerator;

#[derive(Debug)]
struct AttrGen<'a> {
    item: &'a ItemStruct,
    args: &'a Args,
}

impl<'a> AttrGen<'a> {
    fn new(item: &'a ItemStruct, args: &'a Args) -> Self {
        Self { item, args }
    }
}

impl<'a> AttrGenerator for AttrGen<'a> {
    fn error_action_text(&self) -> String {
        format!("generating {} attrs", self.item.ident)
    }

    fn new_non_doc_attrs(&self) -> Vec<Meta> {
        use Attrs::*;

        let original_attrs_it = self
            .item
            .attrs
            .iter()
            // this filter is required, otherwise multiple #[optfield] attributes
            // on the same struct would error
            .filter(|attr| !is_optfield_attr(attr))
            .filter_map(|attr| (!is_doc_attr(attr)).then(|| attr.meta.clone()));

        match &self.args.attrs {
            None => vec![],
            Some(Keep) => original_attrs_it.collect(),
            Some(Replace(attrs)) => attrs.clone(),
            Some(Add(attrs)) => original_attrs_it.chain(attrs.clone()).collect(),
        }
    }

    fn new_doc_attrs(&self) -> Vec<Meta> {
        use Doc::*;

        let original_doc_it = self
            .item
            .attrs
            .iter()
            .filter_map(|attr| is_doc_attr(attr).then(|| attr.meta.clone()));

        match &self.args.doc {
            None => vec![],
            Some(Keep) => original_doc_it.collect(),
            Some(Replace(doc_text)) => {
                let tokens = quote! { doc = #doc_text };
                let doc_attr = parse2(tokens)
                    .unwrap_or_else(|e| panic!("{}", unexpected(self.error_action_text(), e)));
                vec![doc_attr]
            }
            Some(Append(doc_text)) => {
                let tokens = quote! { doc = #doc_text };
                let doc_attr = parse2(tokens)
                    .unwrap_or_else(|e| panic!("{}", unexpected(self.error_action_text(), e)));
                original_doc_it.chain(std::iter::once(doc_attr)).collect()
            }
        }
    }
}

pub fn generate(item: &ItemStruct, args: &Args) -> Vec<Attribute> {
    AttrGen::new(item, args).generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::*;

    #[test]
    fn keep_same_docs() {
        let item = parse_item(quote! {
            /// some
            /// docs
            #[some_attr]
            struct S;
        });

        let cases = vec![
            quote! {
                Opt,
                doc
            },
            quote! {
                Opt,
                doc,
                attrs
            },
            quote! {
                Opt,
                doc,
                attrs = (new_attr)
            },
            quote! {
                Opt,
                doc,
                attrs = add(new_attr)
            },
        ];

        let item_docs = doc_attrs(&item.attrs);

        for case in cases {
            let args = parse_args(case);

            let generated = generate(&item, &args);

            assert!(attrs_contain_all(&generated, &item_docs));
        }
    }

    #[test]
    fn remove_docs() {
        let item = parse_item(quote! {
            /// some
            /// docs
            #[some_attr]
            struct S;
        });

        let cases = vec![
            quote! {
                Opt
            },
            quote! {
                Opt,
                attrs
            },
            quote! {
                Opt,
                attrs = (new_attr)
            },
            quote! {
                Opt,
                attrs = add(new_attr)
            },
        ];

        let item_docs = doc_attrs(&item.attrs);

        for case in cases {
            let args = parse_args(case);

            let generated = generate(&item, &args);

            assert!(!attrs_contain_any(&generated, &item_docs));
        }
    }

    #[test]
    fn replace_docs() {
        let item = parse_item(quote! {
            /// some
            /// old
            /// docs
            #[some_attr]
            struct S;
        });

        let new_doc_text = r#"docs
            that will
            replace the old
            ones"#;

        let cases = vec![
            quote! {
                Opt,
                doc = #new_doc_text
            },
            quote! {
                Opt,
                doc = #new_doc_text,
                attrs
            },
            quote! {
                Opt,
                doc = #new_doc_text,
                attrs = (new_attr),
            },
            quote! {
                Opt,
                doc = #new_doc_text,
                attrs = add(new_attr),
            },
        ];

        let new_doc_attr = parse_attr(quote! {
            #[doc = #new_doc_text]
        });

        let item_docs = doc_attrs(&item.attrs);

        for case in cases {
            let args = parse_args(case);

            let generated = generate(&item, &args);

            assert!(!attrs_contain_any(&generated, &item_docs));
            assert!(generated.contains(&new_doc_attr));
        }
    }

    #[test]
    fn remove_attrs() {
        let (item, args) = parse_item_and_args(
            quote! {
                #[some]
                #[attrs]
                struct S;
            },
            quote! {
                Opt
            },
        );

        let generated = generate(&item, &args);

        assert!(!attrs_contain_any(&generated, &item.attrs));
    }

    #[test]
    fn keep_attrs() {
        let (item, args) = parse_item_and_args(
            quote! {
                #[some]
                #[attrs]
                struct S;
            },
            quote! {
                Opt,
                attrs
            },
        );

        let generated = generate(&item, &args);

        assert!(attrs_contain_all(&generated, &item.attrs));
    }

    #[test]
    fn replace_attrs() {
        let (item, args) = parse_item_and_args(
            quote! {
                #[some]
                #[attrs]
                struct S;
            },
            quote! {
                Opt,
                attrs = (
                    other
                    attributes
                )
            },
        );

        let new_attrs = parse_attrs(quote! {
            #[other]
            #[attributes]
        });

        let generated = generate(&item, &args);

        assert!(!attrs_contain_any(&generated, &item.attrs));
        assert!(attrs_contain_all(&generated, &new_attrs));
    }

    #[test]
    fn add_attrs() {
        let (item, args) = parse_item_and_args(
            quote! {
                #[some]
                #[attrs]
                struct S;
            },
            quote! {
                Opt,
                attrs = add(
                    other
                    attributes
                )
            },
        );

        let new_attrs = parse_attrs(quote! {
            #[other]
            #[attributes]
        });

        let generated = generate(&item, &args);

        assert!(attrs_contain_all(&generated, &item.attrs));
        assert!(attrs_contain_all(&generated, &new_attrs));
    }

    #[test]
    fn remove_other_optfield_attrs() {
        let optfield_attr1 = parse_attr(quote! {
            #[optfield(Opt1, attrs)]
        });

        let optfield_attr2 = parse_attr(quote! {
            #[optfield(Opt2, attrs, doc)]
        });

        let item = parse_item(quote! {
            #[attr]
            #optfield_attr1
            #optfield_attr2
            struct S;
        });

        let cases = vec![
            quote! {
                Opt,
                attrs
            },
            quote! {
                Opt,
                attrs = add(new)
            },
        ];

        for case in cases {
            let args = parse_args(case);

            let generated = generate(&item, &args);

            assert!(!generated.contains(&optfield_attr1));
            assert!(!generated.contains(&optfield_attr2));
        }
    }
}
