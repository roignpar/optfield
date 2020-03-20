use syn::{Attribute, Field};

use crate::args::{Args, Attrs};
use crate::attrs::generator::AttrGenerator;

struct FieldAttrGen<'a> {
    field: &'a Field,
    args: &'a Args,
}

impl<'a> FieldAttrGen<'a> {
    fn new(field: &'a Field, args: &'a Args) -> Self {
        Self { field, args }
    }
}

impl<'a> AttrGenerator for FieldAttrGen<'a> {
    fn no_docs(&self) -> bool {
        !self.args.field_docs
    }

    fn error_action_text(&self) -> String {
        let field_name = match &self.field.ident {
            None => "".to_string(),
            Some(ident) => format!("{} ", ident),
        };

        format!("generating {}field attrs", field_name)
    }

    fn original_attrs(&self) -> &[Attribute] {
        &self.field.attrs
    }

    fn attrs_arg(&self) -> &Option<Attrs> {
        &self.args.field_attrs
    }
}

pub fn generate(field: &mut Field, args: &Args) {
    field.attrs = {
        let generator = FieldAttrGen::new(field, args);

        generator.generate()
    };
}
