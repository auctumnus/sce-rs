#![warn(clippy::pedantic)]
#![allow(clippy::enum_glob_use)]
#![feature(test)]

use ariadne::{sources, Label, Report};
use chumsky::prelude::*;
use parse::AST;

pub mod apply;
pub mod common;
pub mod parse;
pub mod word;

/// Parses source code into an SCE AST.
///
/// ## Panics
/// Panics if it fails to make error reports.
///
/// ## Returns
/// Either the AST or the errors encountered.
///
/// ## Errors
/// Returns parse errors.
pub fn parse(source: &str) -> Result<AST, Vec<Rich<char>>> {
    let (ast, errs) = parse::ast().parse(source).into_output_errors();
    if let Some(ast) = ast {
        println!("ast: {ast:?}");
        return Ok(ast);
    }
    errs.clone()
        .into_iter()
        .map(|e| e.map_token(|c| c.to_string()))
        .for_each(|e| {
            Report::build(
                ariadne::ReportKind::Error,
                String::from("src"),
                e.span().start,
            )
            .with_message(format!("{e:?}"))
            .with_label(
                Label::new((String::from("src"), e.span().into_range()))
                    .with_message(format!("{e:?}")),
            )
            .finish()
            .print(sources([(String::from("src"), source)]))
            .unwrap();
        });

    Err(errs)
}
