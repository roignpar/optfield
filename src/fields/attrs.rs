use quote::quote;
use syn::{parse::Parser, Attribute, Field, Meta};

use crate::args::{Args, Attrs};

use crate::docs::is_doc_attr;
use crate::error::unexpected;

pub fn generate(field: &mut Field, args: &Args) {
    use Attrs::*;

    // no docs and no attrs => remove all attrs
    if !args.field_docs && args.field_attrs.is_none() {
        field.attrs = Vec::new();
        return;
    }

    let new_cap = compute_capacity(field, args);
    let mut new_attrs = Vec::with_capacity(new_cap);

    for attr in &field.attrs {
        let mut add_attr = false;

        if args.field_docs && is_doc_attr(attr) {
            add_attr = true;
        }

        if let Some(Keep) | Some(Add(_)) = args.field_attrs {
            if !is_doc_attr(attr) {
                add_attr = true;
            }
        }

        if add_attr {
            let meta = parse_meta(field, attr);

            new_attrs.push(meta);
        }
    }

    if let Some(Replace(v)) | Some(Add(v)) = &args.field_attrs {
        new_attrs.extend(v.clone());
    }

    let parser = Attribute::parse_outer;
    let attrs_stream = quote! {
        #(#[#new_attrs])*
    };

    field.attrs = parser
        .parse2(attrs_stream)
        .unwrap_or_else(|e| panic!(unexpected_error(field, e)));
}

fn parse_meta(field: &Field, attr: &Attribute) -> Meta {
    attr.parse_meta()
        .unwrap_or_else(|e| panic!(unexpected_error(field, e)))
}

fn compute_capacity(field: &Field, args: &Args) -> usize {
    use Attrs::*;

    match &args.field_attrs {
        None => 0,
        Some(Keep) => field.attrs.len(),
        Some(Replace(v)) => v.len(),
        Some(Add(v)) => v.len() + field.attrs.len(),
    }
}

fn unexpected_error<E>(field: &Field, err: E) -> String
where
    E: std::error::Error + std::fmt::Display,
{
    let field_name = match &field.ident {
        None => "".to_string(),
        Some(ident) => format!("{} ", ident),
    };

    unexpected(format!("generating {}field attrs", field_name), err)
}
