use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::token::{Comma, Eq, Pub};
use syn::{Ident, LitStr, Visibility};

mod kw {
    // NOTE: when adding new keywords update ArgList::next_is_kw
    syn::custom_keyword!(doc);
    syn::custom_keyword!(merge);
    syn::custom_keyword!(rewrap);
    syn::custom_keyword!(field_docs);
}

#[derive(Debug)]
pub struct Args {
    pub name: Ident,
    pub visibility: Option<Visibility>,
    pub merge: Option<MergeFnName>,
    pub rewrap: bool,
    pub doc: Option<Doc>,
    pub field_docs: bool,
}

#[derive(Debug)]
enum Arg {
    Merge(MergeFnName),
    Doc(Doc),
    Rewrap(bool),
    FieldDocs(bool),
    Vis(Visibility),
}

#[derive(Debug)]
pub enum MergeFnName {
    Default,
    Custom(Ident),
}

#[derive(Debug)]
pub enum Doc {
    Same,
    Custom(LitStr),
}

/// Parser for unordered args.
#[derive(Debug)]
struct ArgList {
    struct_name: Ident,
    merge: Option<Span>,
    doc: Option<Span>,
    rewrap: Option<Span>,
    field_docs: Option<Span>,
    visibility: Option<Span>,
    list: Vec<Arg>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let arg_list: ArgList = input.parse()?;

        Ok(arg_list.into())
    }
}

impl Parse for ArgList {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(input.error("expected struct name"));
        }

        if ArgList::next_is_kw(&input) {
            return Err(input.error("first argument must be struct name"));
        }

        let name: Ident = input.parse()?;
        let mut arg_list = ArgList::new(name);

        while !input.is_empty() {
            input.parse::<Comma>()?;

            // allow trailing commas
            if input.is_empty() {
                break;
            }

            let lookahead = input.lookahead1();

            if lookahead.peek(Pub) {
                arg_list.parse_visibility(&input)?;
            } else if lookahead.peek(kw::doc) {
                arg_list.parse_doc(&input)?;
            } else if lookahead.peek(kw::merge) {
                arg_list.parse_merge(&input)?;
            } else if lookahead.peek(kw::rewrap) {
                arg_list.parse_rewrap(&input)?;
            } else if lookahead.peek(kw::field_docs) {
                arg_list.parse_field_docs(&input)?;
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(arg_list)
    }
}

impl Args {
    fn new(name: Ident) -> Self {
        Self {
            name,
            visibility: None,
            merge: None,
            rewrap: false,
            doc: None,
            field_docs: false,
        }
    }
}

impl ArgList {
    fn new(name: Ident) -> Self {
        Self {
            struct_name: name,
            merge: None,
            doc: None,
            rewrap: None,
            field_docs: None,
            visibility: None,
            list: Vec::with_capacity(5),
        }
    }

    fn next_is_kw(input: &ParseStream) -> bool {
        input.peek(Pub)
            || input.peek(kw::doc)
            || input.peek(kw::merge)
            || input.peek(kw::rewrap)
            || input.peek(kw::field_docs)
    }

    fn parse_visibility(&mut self, input: &ParseStream) -> Result<()> {
        if let Some(vis_span) = self.visibility {
            return ArgList::already_defined_error(input, "visibility", vis_span);
        }

        let span = input.span();
        let vis: Visibility = input.parse()?;

        self.visibility = Some(span);
        self.list.push(Arg::Vis(vis));

        Ok(())
    }

    fn parse_doc(&mut self, input: &ParseStream) -> Result<()> {
        if let Some(doc_span) = self.doc {
            return ArgList::already_defined_error(input, "doc", doc_span);
        }

        let span = input.span();
        let doc: Doc = input.parse()?;

        self.doc = Some(span);
        self.list.push(Arg::Doc(doc));

        Ok(())
    }

    fn parse_merge(&mut self, input: &ParseStream) -> Result<()> {
        if let Some(merge_span) = self.merge {
            return ArgList::already_defined_error(input, "merge", merge_span);
        }

        let span = input.span();
        let merge: MergeFnName = input.parse()?;

        self.merge = Some(span);
        self.list.push(Arg::Merge(merge));

        Ok(())
    }

    fn parse_rewrap(&mut self, input: &ParseStream) -> Result<()> {
        if let Some(rewrap_span) = self.rewrap {
            return ArgList::already_defined_error(input, "rewrap", rewrap_span);
        }

        let span = input.span();
        input.parse::<kw::rewrap>()?;

        self.rewrap = Some(span);
        self.list.push(Arg::Rewrap(true));

        Ok(())
    }

    fn parse_field_docs(&mut self, input: &ParseStream) -> Result<()> {
        if let Some(field_docs_span) = self.field_docs {
            return ArgList::already_defined_error(input, "field_docs", field_docs_span);
        }

        let span = input.span();
        input.parse::<kw::field_docs>()?;

        self.field_docs = Some(span);
        self.list.push(Arg::FieldDocs(true));

        Ok(())
    }

    fn already_defined_error(
        input: &ParseStream,
        arg_name: &'static str,
        prev_span: Span,
    ) -> Result<()> {
        let mut e = input.error(&format!("{} already defined", arg_name));
        e.combine(Error::new(
            prev_span,
            &format!("{}, defined here", arg_name),
        ));
        Err(e)
    }
}

impl Parse for Doc {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::doc>()?;

        if input.peek(Eq) {
            input.parse::<Eq>()?;

            let doc_text: LitStr = input.parse()?;

            Ok(Doc::Custom(doc_text))
        } else {
            Ok(Doc::Same)
        }
    }
}

impl Parse for MergeFnName {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::merge>()?;

        if input.peek(Eq) {
            input.parse::<Eq>()?;

            let fn_name: Ident = input.parse()?;

            Ok(MergeFnName::Custom(fn_name))
        } else {
            Ok(MergeFnName::Default)
        }
    }
}

impl From<ArgList> for Args {
    fn from(arg_list: ArgList) -> Args {
        use Arg::*;

        let mut args = Args::new(arg_list.struct_name);

        for arg in arg_list.list {
            match arg {
                Merge(merge_name) => args.merge = Some(merge_name),
                Doc(doc) => args.doc = Some(doc),
                Rewrap(rewrap) => args.rewrap = rewrap,
                FieldDocs(field_docs) => args.field_docs = field_docs,
                Vis(visibility) => args.visibility = Some(visibility),
            }
        }

        args
    }
}
