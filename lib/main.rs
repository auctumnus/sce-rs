#![feature(test)]

use ariadne::{sources, Label, Report};
use chumsky::prelude::*;

mod common;
mod parse;
mod word;

pub fn parse(source: &str) {
    let (ast, errs) = parse::ast().parse(source).into_output_errors();
    if let Some(ast) = ast {
        println!("ast: {ast:?}")
    }
    /*for err in errs {
        println!("{err}")
    }*/
    errs.into_iter()
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
            .unwrap()
        })
    /*
    let (tokens, errs) = tokenize::tokenize().parse(source).into_output_errors();
    if let Some(tokens) = tokens {
        println!("tokens: {tokens:?}")
    }
    println!("errs: {errs:?}")*/
}
