#![doc(html_root_url = "https://docs.rs/optfield/0.3.0")]

//! `optfield` is a macro that, given a struct, generates another struct with
//! the same fields, but wrapped in `Option<T>`.
//!
//! Features:
//! * [Simple examples](#simple-examples)
//! * [Visibility](#visibility)
//! * [Rewrapping `Option` fields](#rewrapping-option-fields)
//! * [Documentation](#documentation)
//! * [Attributes](#attributes)
//! * [Field documentation](#field-documentation)
//! * [Field attributes](#field-attributes)
//! * [Merging](#merging)
//! * [From](#from)
//! * [Custom wrappers](#custom-wrappers)
//!   * [rewrap](#wrapper--rewrap)
//!   * [merging](#wrapper--merge_fn)
//!   * [from](#wrapper--from)
//!   * [nesting](#wrapper-nesting)
//!
//! # Simple examples
//! The first argument is the name of the generated struct:
//! ```
//! use optfield::optfield;
//!
//! #[optfield(Opt)]
//! struct MyStruct {
//!     text: String
//! }
//! ```
//! Will generate the following __second__ struct (leaving `MyStruct` as is):
//! ```
//! struct Opt {
//!     text: Option<String>
//! }
//! ```
//! It also works with tuple structs:
//! ```
//! # use optfield::*;
//! #[optfield(Opt)]
//! struct MyTuple(String, i32);
//! ```
//! Will generate:
//! ```
//! struct Opt(Option<String>, Option<i32>);
//! ```
//! Generics and lifetimes are preserved:
//! ```
//! # use optfield::*;
//! #[optfield(Opt)]
//! struct MyStruct<'a, T> {
//!     field: &'a T
//! }
//! ```
//! Will generate:
//! ```
//! struct Opt<'a, T> {
//!     field: Option<&'a T>
//! }
//! ```
//!
//! # Visibility
//! By default, opt structs are private. To use custom visibility simply add it
//! right before the opt struct name:
//! ```
//! # use optfield::*;
//! #[optfield(pub(crate) Opt)]
//! struct MyStruct {
//!     text: String
//! }
//! ```
//! Will generate:
//! ```
//! pub(crate) struct Opt {
//!     text: Option<String>
//! }
//! ```
//! Field visibility is preserved.
//!
//! # Rewrapping `Option` fields
//! By default, fields that are already wrapped in `Option<T>` are not wrapped
//! again:
//! ```
//! # use optfield::*;
//! #[optfield(Opt)]
//! struct MyStruct {
//!     text: Option<String>,
//!     number: i32
//! }
//! ```
//! Will generate:
//! ```
//! struct Opt {
//!     text: Option<String>,
//!     number: Option<i32>
//! }
//! ```
//! To rewrap them pass the `rewrap` argument:
//!
//! ```
//! # use optfield::*;
//! #[optfield(Opt, rewrap)]
//! struct MyStruct {
//!     text: Option<String>,
//!     number: i32
//! }
//! ```
//! Will generate:
//! ```
//! struct Opt {
//!     text: Option<Option<String>>,
//!     number: Option<i32>
//! }
//! ```
//!
//! # Documentation
//! To document the opt struct, either duplicate the same documentation as the
//! original using the `doc` argument by itself:
//! ```
//! # use optfield::*;
//! /// My struct documentation
//! /// ...
//! # use optfield::*;
//! #[optfield(Opt, doc)]
//! struct MyStruct {
//!     text: String
//! }
//! ```
//! Will generate:
//! ```
//! /// My struct documentation
//! /// ...
//! struct Opt {
//!     text: Option<String>
//! }
//! ```
//! Or write custom documentation by giving `doc` a value:
//! ```
//! # use optfield::*;
//! #[optfield(
//!     Opt,
//!     doc = "
//!         Custom documentation
//!         for Opt struct...
//!     "
//! )]
//! struct MyStruct {
//!     text: String
//! }
//! ```
//! Will generate:
//! ```
//! /// Custom documentation
//! /// for Opt struct...
//! struct Opt {
//!     text: Option<String>
//! }
//! ```
//!
//! # Attributes
//! The `attrs` argument alone makes optfield insert the same attributes as the
//! original:
//! ```
//! # use optfield::*;
//! #[optfield(Opt, attrs)]
//! #[cfg(test)]
//! #[derive(Clone)]
//! struct MyStruct {
//!     text: String
//! }
//! ```
//! Will generate:
//! ```
//! #[cfg(test)]
//! #[derive(Clone)]
//! struct Opt {
//!     text: Option<String>
//! }
//! ```
//! To add more attributes besides the original ones, use `attrs = add(...)`:
//! ```
//! # use optfield::*;
//! #[optfield(
//!     Opt,
//!     attrs = add(
//!         cfg(test),
//!         derive(Clone)
//!     )
//! )]
//! #[derive(Debug)]
//! struct MyStruct {
//!     text: String
//! }
//! ```
//! Will generate:
//! ```
//! #[derive(Debug)]
//! #[cfg(test)]
//! #[derive(Clone)]
//! struct Opt {
//!     text: Option<String>
//! }
//! ```
//! To replace with other attributes, `attrs = (...)`:
//! ```
//! # use optfield::*;
//! #[optfield(
//!     Opt,
//!     attrs = (
//!         cfg(test),
//!         derive(Clone)
//!     )
//! )]
//! #[derive(Debug)]
//! struct MyStruct {
//!     text: String
//! }
//! ```
//! Will generate:
//! ```
//! #[cfg(test)]
//! #[derive(Clone)]
//! struct Opt {
//!     text: Option<String>
//! }
//! ```
//! **NOTE** on attribute order: `optfield`, like any other proc macro, only
//! sees the attributes defined after it:
//!
//! ```
//! # use optfield::*;
//! #[cfg(test)] // optfield is unaware of this attribute
//! #[optfield(Opt, attrs)]
//! #[derive(Debug)]
//! struct MyStruct;
//! ```
//! Will generate:
//! ```
//! #[derive(Debug)]
//! struct Opt;
//! ```
//!
//! # Field documentation
//! By default, field documentation is removed:
//! ```
//! # use optfield::*;
//! #[optfield(Opt)]
//! struct MyStruct {
//!     /// Field
//!     /// documentation
//!     text: String
//! }
//! ```
//! Will generate:
//! ```
//! struct Opt {
//!     text: Option<String>
//! }
//! ```
//! To preserve field documentation use the `field_doc` argument:
//! ```
//! # use optfield::*;
//! #[optfield(Opt, field_doc)]
//! struct MyStruct {
//!     /// Field
//!     /// documentation
//!     text: String
//! }
//! ```
//! Will generate:
//! ```
//! struct Opt {
//!     /// Field
//!     /// documentation
//!     text: Option<String>
//! }
//! ```
//!
//! # Field attributes
//! Field attributes can be handled using the `field_attrs` argument which works
//! similarly to `attrs`, but applies to all fields.
//!
//! `field_attrs` can be used independently of `attrs`.
//!
//! By default, no field attributes are inserted:
//! ```
//! # use optfield::*;
//! # use serde::Deserialize;
//! #[optfield(Opt, attrs)]
//! #[derive(Deserialize)]
//! struct MyStruct {
//!     #[serde(rename = "text")]
//!     my_text: String
//! }
//! ```
//! Will generate:
//! ```
//! # use serde::Deserialize;
//! #[derive(Deserialize)]
//! struct Opt {
//!     my_text: Option<String>
//! }
//! ```
//! To keep them:
//! ```
//! # use optfield::*;
//! # use serde::Deserialize;
//! #[optfield(Opt, attrs, field_attrs)]
//! #[derive(Deserialize)]
//! struct MyStruct {
//!     #[serde(rename = "text")]
//!     my_text: String
//! }
//! ```
//! Will generate:
//! ```
//! # use serde::Deserialize;
//! #[derive(Deserialize)]
//! struct Opt {
//!     #[serde(rename = "text")]
//!     my_text: Option<String>
//! }
//! ```
//! To add more attributes:
//! ```
//! # use optfield::*;
//! # use serde::Deserialize;
//! #[optfield(
//!     Opt,
//!     attrs,
//!     field_attrs = add(
//!         serde(default)
//!     )
//! )]
//! #[derive(Deserialize)]
//! struct MyStruct {
//!     #[serde(rename = "text")]
//!     my_text: String,
//!     #[serde(rename = "number")]
//!     my_number: i32
//! }
//! ```
//! Will generate:
//! ```
//! # use serde::Deserialize;
//! #[derive(Deserialize)]
//! struct Opt {
//!     #[serde(rename = "text")]
//!     #[serde(default)]
//!     my_text: Option<String>,
//!     #[serde(rename = "number")]
//!     #[serde(default)]
//!     my_number: Option<i32>
//! }
//! ```
//! To replace all field attributes:
//! ```
//! # use optfield::*;
//! # use serde::Deserialize;
//! #[optfield(
//!     Opt,
//!     attrs,
//!     field_attrs = (
//!         serde(default)
//!     )
//! )]
//! #[derive(Deserialize)]
//! struct MyStruct {
//!     #[serde(rename = "text")]
//!     my_text: String,
//!     #[serde(rename = "number")]
//!     my_number: i32
//! }
//! ```
//! Will generate:
//! ```
//! # use serde::Deserialize;
//! #[derive(Deserialize)]
//! struct Opt {
//!     #[serde(default)]
//!     my_text: Option<String>,
//!     #[serde(default)]
//!     my_number: Option<i32>
//! }
//! ```
//!
//! # Merging
//! When the  `merge_fn` argument is used `optfield` will add a method to the
//! original struct that merges an opt struct back into the original.
//!
//! By default, the method is named `merge_opt` and has the following signature:
//! ```
//! # struct Opt;
//! # impl Opt {
//! // assuming the opt struct is named Opt;
//! // takes opt by value;
//! fn merge_opt(&mut self, opt: Opt)
//! # {}
//! # }
//! ```
//! When merging, all values of the opt struct that are `Some(...)` are set as
//! values of the original struct fields.
//!
//! To use it:
//! ```
//! # use optfield::*;
//! #[optfield(Opt, merge_fn)]
//! struct MyStruct {
//!     text: String,
//!     number: i32
//! }
//!
//! let mut original = MyStruct {
//!     text: "awesome".to_string(),
//!     number: 1
//! };
//!
//! let opt = Opt {
//!     text: Some("amazing".to_string()),
//!     number: None
//! };
//!
//! original.merge_opt(opt);
//!
//! // text field value is merged
//! assert_eq!(original.text, "amazing");
//! // number field stays the same
//! assert_eq!(original.number, 1);
//! ```
//! The merge function can be given:
//! * custom name: `merge_fn = my_merge_fn`
//! * custom visibility (default is private): `merge_fn = pub(crate)`
//! * both: `merge_fn = pub my_merge_fn`
//!
//! # From
//! When the `from` argument is used, `From<MyStruct>` is implemented for `Opt`.
//! ```
//! # use optfield::*;
//! #[optfield(Opt, from)]
//! struct MyStruct {
//!     text: String,
//!     number: i32,
//! }
//!
//! let original = MyStruct {
//!     text: "super".to_string(),
//!     number: 2,
//! };
//!
//! let from = Opt::from(original);
//! assert_eq!(from.text.unwrap(), "super");
//! assert_eq!(from.number.unwrap(), 2);
//! ```
//!
//! # Custom wrappers
//! Custom wrappers can be used instead of `Option` with the `wrapper` argument:
//! ```
//! # use optfield::*;
//! enum MyWrapper<T> {
//!     Empty,
//!     Number(i32),
//!     Something(T),
//! }
//!
//! #[optfield(Opt, wrapper = MyWrapper)]
//! struct MyStruct {
//!     optional: Option<i32>,
//!     text: String,
//! }
//! ```
//! Will generate:
//! ```
//! # struct MyWrapper<T>(T);
//! struct Opt {
//!     optional: MyWrapper<Option<i32>>,
//!     text: MyWrapper<String>,
//! }
//! ```
//! Any type with a signature taking exactly one generic (`Wrapper<T>`) can be
//! used as a wrapper: structs, tuples, enums or type aliases.
//!
//! When using `wrapper` some behavior changes:
//!
//! ### `wrapper` + `rewrap`
//! `rewrap` will target the wrapper instead of `Option`:
//! ```
//! # use optfield::*;
//! # struct MyWrapper<T>(T);
//! #[optfield(Opt, wrapper = MyWrapper)]
//! struct MyStruct {
//!     optional: Option<i32>,
//!     text: String,
//!     wrapped: MyWrapper<i32>,
//! }
//! ```
//! Will generate:
//! ```
//! # struct MyWrapper<T>(T);
//! struct Opt {
//!     optional: MyWrapper<Option<i32>>,
//!     text: MyWrapper<String>,
//!     wrapped: MyWrapper<String>,
//! }
//! ```
//! Whereas
//! ```
//! # use optfield::*;
//! # struct MyWrapper<T>(T);
//! #[optfield(Opt, wrapper = MyWrapper, rewrap)]
//! struct MyStruct {
//!     optional: Option<i32>,
//!     text: String,
//!     wrapped: MyWrapper<i32>,
//! }
//! ```
//! Will generate:
//! ```
//! # struct MyWrapper<T>(T);
//! struct Opt {
//!     optional: MyWrapper<Option<i32>>,
//!     text: MyWrapper<String>,
//!     wrapped: MyWrapper<MyWrapper<String>>,
//! }
//! ```
//! ### `wrapper` + `merge_fn`
//! Unlike with `Option` which is part of Rust core, optfield
//! can't make any assumptions about the custom types used as wrappers, so
//! when using `merge_fn` a few extra things are required.
//!
//! A `From<MyWrapper<T>> for Option<T>` implementation must exist, either
//! generically or specifically for each type used for the original struct's
//! fields. For each field the value will be merged when
//! `Option::<T>::from(opt.field)` is `Some(...)` and ignored when it is `None`,
//! `T` being the field's type in the original struct and `opt` an instance of
//! the struct generated by optfield. Example:
//! ```
//! # use optfield::*;
//! enum MyWrapper<T> {
//!     Nothing,
//!     NothingElse,
//!     Something(T),
//! }
//!
//! impl<T> From<MyWrapper<T>> for Option<T> {
//!     fn from(value: MyWrapper<T>) -> Option<T> {
//!         match value {
//!             MyWrapper::Nothing => None,
//!             MyWrapper::NothingElse => None,
//!             MyWrapper::Something(t) => Some(t),
//!         }
//!     }
//! }
//!
//! #[optfield(Opt, wrapper = MyWrapper, merge_fn)]
//! struct MyStruct {
//!     number: i32,
//!     another_number: u32,
//!     text: String,
//! }
//!
//! let mut original = MyStruct {
//!     number: 0,
//!     another_number: 0,
//!     text: "nice".to_string(),
//! };
//!
//! let opt = Opt {
//!     number: MyWrapper::Nothing,
//!     another_number: MyWrapper::NothingElse,
//!     text: MyWrapper::Something("great".to_string()),
//! };
//!
//! original.merge_opt(opt);
//!
//! assert_eq!(original.number, 0);
//! assert_eq!(original.another_number, 0);
//! assert_eq!(original.text, "great".to_string());
//! ```
//! Furthermore, if any of the original fields are already wrapped in the custom
//! wrapper and you are **not** using `rewrap`, a
//! `From<&MyWrapper<T>> for Option<()>` is required. This is needed in order to
//! figure out whether to merge the value without turning it into an `Option`
//! and losing its type information. If `Option::<()>::from(opt.field)` is
//! `Some(())` then the value in the original struct instance will be replaced
//! with the one from the wrapped struct and otherwise ignored. Example:
//! ```
//! # use optfield::*;
//! #[derive(Debug, PartialEq)]
//! enum MyWrapper<T> {
//!     Nothing,
//!     Something(T),
//! }
//!
//! impl<T> From<MyWrapper<T>> for Option<T> {
//!     fn from(value: MyWrapper<T>) -> Option<T> {
//!         match value {
//!             MyWrapper::Nothing => None,
//!             MyWrapper::Something(t) => Some(t),
//!         }
//!     }
//! }
//!
//! impl<T> From<&MyWrapper<T>> for Option<()> {
//!     fn from(value: &MyWrapper<T>) -> Self {
//!         match value {
//!             MyWrapper::Nothing => None,
//!             MyWrapper::Something(_) => Some(()),
//!         }
//!     }
//! }
//!
//! #[optfield(Opt, wrapper = MyWrapper, merge_fn)]
//! struct MyStruct {
//!     number: i32,
//!     wrapped: MyWrapper<i32>,
//! }
//!
//! let mut original = MyStruct {
//!     number: 0,
//!     wrapped: MyWrapper::Something(0),
//! };
//!
//! let opt = Opt {
//!     number: MyWrapper::Something(42),
//!     wrapped: MyWrapper::Something(1337),
//! };
//!
//! original.merge_opt(opt);
//!
//! assert_eq!(original.number, 42);
//! assert_eq!(original.wrapped, MyWrapper::Something(1337));
//! ```
//! ### `wrapper` + `from`
//! When using `from` a `From<T> for MyWrapper<T>` implementation must exist,
//! either generic or specific for each of the target struct field types:
//! ```
//! # use optfield::*;
//! #[derive(Debug, PartialEq)]
//! enum MyWrapper<T> {
//!     Nothing,
//!     Something(T),
//! }
//!
//! impl<T> From<T> for MyWrapper<T> {
//!     fn from(value: T) -> Self {
//!         MyWrapper::Something(value)
//!     }
//! }
//!
//! #[optfield(Opt, wrapper = MyWrapper, from)]
//! struct MyStruct {
//!     field: i32,
//! }
//!
//! let original = MyStruct { field: 88 };
//!
//! let opt = Opt::from(original);
//!
//! assert_eq!(opt.field, MyWrapper::Something(88));
//! ```
//! ### Wrapper nesting
//! With its support for attributes optfield can be nested. This may be useful
//! to combine various wrappers.
//! ```
//! # use optfield::*;
//! struct MyWrapper<T>(T);
//!
//! #[optfield(OptMyWrapper, wrapper = MyWrapper, attrs = (
//!     optfield(OptOption)
//! ))]
//! struct MyStruct {
//!     number: i32,
//!     text: String,
//! }
//! ```
//! Will generate:
//! ```
//! # struct MyWrapper<T>(T);
//! struct OptMyWrapper {
//!     number: MyWrapper<i32>,
//!     text: MyWrapper<String>,
//! }
//!
//! struct OptOption {
//!     number: Option<MyWrapper<i32>>,
//!     text: Option<MyWrapper<String>>,
//! }
//! ```
extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

mod args;
mod attrs;
mod error;
mod fields;
mod from;
mod generate;
mod merge;

use args::Args;
use generate::generate;

/// The macro
///
/// Check [crate documentation] for more information.
///
/// [crate documentation]: ./index.html
#[proc_macro_attribute]
pub fn optfield(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: ItemStruct = parse_macro_input!(item);
    let args: Args = parse_macro_input!(attr);

    let opt_item = generate(&item, args);

    let out = quote! {
        #item

        #opt_item
    };

    out.into()
}

#[cfg(test)]
mod test_util {
    use proc_macro2::TokenStream;
    use syn::{parse::Parser, parse2, Attribute, Field, Fields, ItemStruct, Type, Visibility};

    use crate::args::Args;
    use crate::attrs::generator::is_doc_attr;

    pub fn parse_item_and_args(
        item_tokens: TokenStream,
        args_tokens: TokenStream,
    ) -> (ItemStruct, Args) {
        (parse_item(item_tokens), parse_args(args_tokens))
    }

    pub fn parse_field_and_args(
        field_tokens: TokenStream,
        args_tokens: TokenStream,
    ) -> (Field, Args) {
        (parse_field(field_tokens), parse_args(args_tokens))
    }

    pub fn parse_item(tokens: TokenStream) -> ItemStruct {
        parse2(tokens).unwrap()
    }

    pub fn parse_field(tokens: TokenStream) -> Field {
        Field::parse_named.parse2(tokens).unwrap()
    }

    pub fn parse_args(tokens: TokenStream) -> Args {
        parse2(tokens).unwrap()
    }

    pub fn parse_attr(tokens: TokenStream) -> Attribute {
        parse_attrs(tokens).get(0).unwrap().clone()
    }

    pub fn parse_attrs(tokens: TokenStream) -> Vec<Attribute> {
        Attribute::parse_outer.parse2(tokens).unwrap()
    }

    pub fn parse_type(tokens: TokenStream) -> Type {
        parse2(tokens).unwrap()
    }

    pub fn parse_types(tokens: Vec<TokenStream>) -> Vec<Type> {
        tokens.into_iter().map(parse_type).collect()
    }

    pub fn field_types(fields: Fields) -> Vec<Type> {
        fields.iter().map(|f| f.ty.clone()).collect()
    }

    pub fn parse_visibility(tokens: TokenStream) -> Visibility {
        parse2(tokens).unwrap()
    }

    pub fn attrs_contain_all(attrs: &[Attribute], other_attrs: &[Attribute]) -> bool {
        for attr in other_attrs {
            if !attrs.contains(attr) {
                return false;
            }
        }

        true
    }

    pub fn attrs_contain_any(attrs: &[Attribute], any_attrs: &[Attribute]) -> bool {
        for attr in any_attrs {
            if attrs.contains(attr) {
                return true;
            }
        }

        false
    }

    pub fn doc_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
        attrs
            .iter()
            .filter(|a| is_doc_attr(a))
            .map(|a| a.clone())
            .collect()
    }
}
