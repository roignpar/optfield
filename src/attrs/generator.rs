use quote::quote;
use syn::{parse::Parser, Attribute, Meta};

use crate::error;

const DOC: &str = "doc";
const OPTFIELD_ATTR_NAME: &str = "optfield";

pub trait AttrGenerator {
    fn error_action_text(&self) -> String;

    fn new_non_doc_attrs(&self) -> Vec<Meta>;

    fn new_doc_attrs(&self) -> Vec<Meta>;

    fn generate(&self) -> Vec<Attribute> {
        let attrs_except_docs = self.new_non_doc_attrs();
        let docs = self.new_doc_attrs();

        let attrs_tokens = quote! {
            #(#[#attrs_except_docs])*
            #(#[#docs])*
        };

        Attribute::parse_outer
            .parse2(attrs_tokens)
            .unwrap_or_else(|e| panic!("{}", error::unexpected(self.error_action_text(), e)))
    }
}

/// Useful during attribute generation: it can prevent some unwanted recursive
/// behaviour.
pub fn is_optfield_attr(attr: &Attribute) -> bool {
    attr.path().is_ident(OPTFIELD_ATTR_NAME)
}

pub fn is_doc_attr(attr: &Attribute) -> bool {
    attr.path().is_ident(DOC)
}
