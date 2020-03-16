use quote::quote;
use syn::{parse::Parser, Attribute, ItemStruct, LitStr};

use crate::args::{Args, Doc};
use crate::error::unexpected;

const DOC: &str = "doc";

/// Changes item in place.
/// If args.doc is None, removes doc attrs from item.
/// If args.doc is Some(Same), does nothing.
/// If args.doc is Some(Custom(docs)), sets doc attrs to docs.
pub fn generate(item: &mut ItemStruct, args: &Args) {
    use Doc::*;

    match &args.doc {
        Some(Same) => {}
        None => remove_doc_attrs(&mut item.attrs),
        Some(Custom(docs)) => replace_doc_attrs(item, docs),
    }
}

fn remove_doc_attrs(attrs: &mut Vec<Attribute>) {
    let mut new_attrs = Vec::with_capacity(attrs.len());

    for attr in attrs.iter() {
        if !is_doc_attr(attr) {
            new_attrs.push(attr.clone());
        }
    }

    *attrs = new_attrs;
}

fn replace_doc_attrs(item: &mut ItemStruct, docs: &LitStr) {
    remove_doc_attrs(&mut item.attrs);

    let doc_string = docs.value();
    let lines = doc_string.lines();

    let parser = Attribute::parse_outer;
    let doc_attrs = quote! {
        #(#[doc = #lines])*
    };

    item.attrs = parser
        .parse2(doc_attrs)
        .unwrap_or_else(|e| panic!(unexpected(format!("generating {} docs", item.ident), e)));
}

pub fn is_doc_attr(attr: &Attribute) -> bool {
    attr.path.is_ident(DOC)
}
