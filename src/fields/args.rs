use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream, Result},
    token::Comma,
    Error,
};

use crate::args::{kw, Attrs, Doc};

/// Args declared on a field.
#[derive(Debug)]
pub struct FieldArgs {
    pub doc: Option<Doc>,
    pub attrs: Option<Attrs>,
}
impl Parse for FieldArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let arg_list: FieldArgList = input.parse()?;

        Ok(arg_list.into())
    }
}
impl FieldArgs {
    fn new() -> Self {
        Self {
            doc: None,
            attrs: None,
        }
    }
}

#[derive(Debug)]
enum FieldArg {
    Doc(Doc),
    Attrs(Attrs),
}

/// Parser for unordered args on a field.
#[derive(Debug)]
struct FieldArgList {
    doc: Option<Span>,
    attrs: Option<Span>,
    list: Vec<FieldArg>,
}
impl Parse for FieldArgList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut arg_list = FieldArgList::new();

        while !input.is_empty() {
            input.parse::<Comma>()?;

            // allow trailing commas
            if input.is_empty() {
                break;
            }

            let lookahead = input.lookahead1();

            if lookahead.peek(kw::doc) {
                arg_list.parse_doc(input)?;
            } else if lookahead.peek(kw::attrs) {
                arg_list.parse_attrs(input)?;
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(arg_list)
    }
}
impl From<FieldArgList> for FieldArgs {
    fn from(arg_list: FieldArgList) -> Self {
        use FieldArg::*;

        let mut args = FieldArgs::new();

        for arg in arg_list.list {
            match arg {
                Doc(doc) => args.doc = Some(doc),
                Attrs(attrs) => args.attrs = Some(attrs),
            }
        }

        args
    }
}
impl FieldArgList {
    fn new() -> Self {
        Self {
            doc: None,
            attrs: None,
            list: Vec::with_capacity(2),
        }
    }

    fn parse_doc(&mut self, input: ParseStream) -> Result<()> {
        if let Some(doc_span) = self.doc {
            return FieldArgList::already_defined_error(input, "doc", doc_span);
        }

        let span = input.span();
        let doc: Doc = input.parse()?;

        self.doc = Some(span);
        self.list.push(FieldArg::Doc(doc));

        Ok(())
    }

    fn parse_attrs(&mut self, input: ParseStream) -> Result<()> {
        if let Some(attrs_span) = self.attrs {
            return FieldArgList::already_defined_error(input, "attrs", attrs_span);
        }

        let span = input.span();

        input.parse::<kw::attrs>()?;
        let attrs: Attrs = input.parse()?;

        self.attrs = Some(span);
        self.list.push(FieldArg::Attrs(attrs));

        Ok(())
    }

    fn already_defined_error(
        input: ParseStream,
        arg_name: &'static str,
        prev_span: Span,
    ) -> Result<()> {
        let mut e = input.error(format!("{} already defined", arg_name));
        e.combine(Error::new(prev_span, format!("{} defined here", arg_name)));
        Err(e)
    }
}
