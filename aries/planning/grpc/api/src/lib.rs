#![allow(non_camel_case_types)]
#![allow(clippy::derive_partial_eq_without_eq)]
#[rustfmt::skip]
mod unified_planning;

// Re-exports all items from the unified_planning module, that is automatically
// generated by prost from the protobuf definition.
pub use crate::unified_planning::*;

use crate::atom::Content;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(atom) = &self.atom {
            match atom.content.as_ref().unwrap() {
                Content::Symbol(s) => write!(f, "{s}"),
                Content::Int(i) => write!(f, " {i}"),
                Content::Real(r) => {
                    let float = (r.numerator as f32) / (r.denominator as f32);
                    write!(f, "{float}")
                }
                Content::Boolean(b) => write!(f, "{b}"),
            }
        } else {
            write!(f, "(")?;
            write!(f, "{}", self.list.iter().format(" "))?;
            write!(f, ")")
        }
    }
}