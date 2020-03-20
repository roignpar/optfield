use quote::quote;
use syn::{parse2, Attribute, ItemStruct, Meta};

use crate::args::{Args, Attrs, Doc};
use crate::error::unexpected;

pub mod generator;

use generator::AttrGenerator;

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
    fn no_docs(&self) -> bool {
        self.args.doc.is_none()
    }

    fn error_action_text(&self) -> String {
        format!("generating {} attrs", self.item.ident)
    }

    fn original_attrs(&self) -> &[Attribute] {
        &self.item.attrs
    }

    fn attrs_arg(&self) -> &Option<Attrs> {
        &self.args.attrs
    }

    fn custom_docs(&self) -> Option<Meta> {
        if let Some(Doc::Custom(d)) = &self.args.doc {
            let tokens = quote! {
                doc = #d
            };

            Some(parse2(tokens).unwrap_or_else(|e| panic!(unexpected(self.error_action_text(), e))))
        } else {
            None
        }
    }

    fn keep_original_docs(&self) -> bool {
        use Doc::*;

        match self.args.doc {
            None | Some(Custom(_)) => false,
            Some(Same) => true,
        }
    }
}

pub fn generate(item: &ItemStruct, args: &Args) -> Vec<Attribute> {
    AttrGen::new(item, args).generate()
}
