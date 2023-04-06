use chumsky::{
    prelude::*,
    text::{digits, inline_whitespace, newline, whitespace},
};
use std::str::FromStr;
use strum::EnumString;

use crate::common::Wildcard;

const CONTROL_CHARACTERS: &str = "[]{}<>()@!%^_, *?\\+-^/=";

type E<'a> = extra::Err<Rich<'a, char, SimpleSpan<usize>>>;

#[derive(Clone, Debug, PartialEq)]
pub enum CatOrEl {
    Cat(String),
    El(String),
}

#[derive(Clone, Debug)]
pub enum CategoryEditKind {
    Def,
    Add,
    Sub,
}

#[derive(Clone, Debug)]
pub struct CategoryEdit {
    target: String,
    elements: Vec<CatOrEl>,
    kind: CategoryEditKind,
}

fn escape<'a>() -> impl Parser<'a, &'a str, char, E<'a>> {
    just('\\').ignore_then(one_of(CONTROL_CHARACTERS))
}

#[cfg(test)]
#[test]
fn escape_test() {
    assert_eq!(
        escape().parse("\\[").into_output_errors(),
        (Some('['), vec![])
    );
}

fn text<'a>() -> impl Parser<'a, &'a str, String, E<'a>> {
    none_of(CONTROL_CHARACTERS)
        .and_is(whitespace().at_least(1).not())
        .and_is(escape().not())
        .or(escape())
        .repeated()
        .at_least(1)
        // TODO: it's a little disgusting that i have to allocate this
        // but i can't just take from the original string
        .collect::<String>()
}

#[cfg(test)]
#[test]
fn text_test() {
    let passing_cases = [("abc", "abc"), ("\\[a\\]", "[a]")];

    passing_cases.into_iter().for_each(|(input, expected)| {
        let (parsed, errs) = text().parse(input).into_output_errors();
        assert_eq!(parsed, Some(String::from(expected)));
        assert!(errs.len() == 0);
    });

    let failing_cases = ["\\", "\\n", "a "];

    failing_cases.into_iter().for_each(|input| {
        let (parsed, errs) = text().parse(input).into_output_errors();
        assert!(parsed.is_none());
        assert!(errs.len() > 0);
    });
}

fn cat_or_els<'a>() -> impl Parser<'a, &'a str, Vec<CatOrEl>, E<'a>> {
    text()
        .delimited_by(just('['), just(']'))
        .map(CatOrEl::Cat)
        .or(text().map(CatOrEl::El))
        .separated_by(just(',').then_ignore(inline_whitespace()))
        .at_least(1)
        .collect::<Vec<_>>()
}

#[cfg(test)]
#[test]
fn cat_or_els_test() {
    use CatOrEl::*;
    assert_eq!(
        cat_or_els().parse("a,b,[c]").into_output(),
        Some(vec![
            El(String::from("a")),
            El(String::from("b")),
            Cat(String::from("c"))
        ])
    );
}

pub fn cat_edit<'a>() -> impl Parser<'a, &'a str, CategoryEdit, E<'a>> {
    let kind = choice((
        just('=').to(CategoryEditKind::Def),
        just("+=").to(CategoryEditKind::Add),
        just("-=").to(CategoryEditKind::Sub),
    ));

    text()
        .then_ignore(inline_whitespace())
        .then(kind)
        .then_ignore(inline_whitespace())
        .then(cat_or_els())
        .map(|((target, kind), elements)| CategoryEdit {
            target,
            kind,
            elements,
        })
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternElement {
    Text(String),
    Optional(Pattern),
    OptionalNonGreedy(Pattern),
    Wildcard(Wildcard),
    RepeatN(usize),
    RepeatWild(Wildcard),
    CatRef(String),
    Category(Vec<CatOrEl>),
    Ditto,
    Target,
    TargetReversed,
}

pub fn pattern_element<'src>(
    pattern: impl Parser<'src, &'src str, Pattern, E<'src>> + Clone,
) -> impl Parser<'src, &'src str, PatternElement, E<'src>> {
    let wildcard_inner =
        choice((just("**?"), just("**"), just("*?"), just("*"))).try_map(|s, span| {
            Wildcard::from_str(s)
                .map_err(|e| Rich::custom(span, format!("couldn't parse wildcard: {e}")))
        });

    let wildcard = wildcard_inner.map(PatternElement::Wildcard);

    let optional_non_greedy = pattern
        .clone()
        .delimited_by(just('('), just(")?"))
        .map(PatternElement::OptionalNonGreedy);

    let optional = pattern
        .delimited_by(just('('), just(')'))
        .map(PatternElement::Optional);

    let repeat_n = digits(10)
        .slice()
        .try_map(|t, span| {
            usize::from_str_radix(t, 10)
                .map_err(|e| Rich::custom(span, "couldn't parse repeat int"))
        })
        .delimited_by(just('{'), just('}'))
        .map(PatternElement::RepeatN);

    let repeat_wild = wildcard_inner
        .delimited_by(just('{'), just('}'))
        .map(PatternElement::RepeatWild);

    let cat_ref = text()
        .delimited_by(just('['), just(']'))
        .map(PatternElement::CatRef);

    let null_category = just("[]").to(PatternElement::Category(vec![]));

    let category = cat_or_els()
        .delimited_by(just('['), just(']'))
        .map(PatternElement::Category);

    let simple = choice((
        just('%').to(PatternElement::Target),
        just('"').to(PatternElement::Ditto),
        just('<').to(PatternElement::TargetReversed),
    ));

    choice((
        optional_non_greedy,
        optional,
        wildcard,
        repeat_wild,
        repeat_n,
        null_category,
        cat_ref,
        category,
        simple,
        text().map(PatternElement::Text),
    ))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pattern {
    elements: Vec<PatternElement>,
}

pub fn pattern<'src>() -> impl Parser<'src, &'src str, Pattern, E<'src>> {
    recursive(|pat| {
        pattern_element(pat)
            .repeated()
            .collect::<Vec<PatternElement>>()
            .map(|elements| Pattern { elements })
            .boxed() // required to avoid an evil type error
    })
}

#[cfg(test)]
#[test]
fn pattern_test() {
    use self::Wildcard::*;
    use PatternElement::*;

    let cases = [
        ("a", vec![Text(String::from("a"))]),
        ("*", vec![Wildcard(Greedy)]),
    ];

    for (input, expected) in cases {
        let actual = pattern().parse(input).into_output().map(|p| p.elements);
        assert_eq!(actual, Some(expected));
    }
}

#[derive(Debug, Clone, Default)]
struct Change {
    pattern: Pattern,
}

fn change<'src>() -> impl Parser<'src, &'src str, Change, E<'src>> {
    pattern().map(|pattern| Change { pattern })
}

/// Groups together environments that are connected via `&`.
#[derive(Debug, Clone, Default)]
struct EnvironmentGroup {
    patterns: Vec<Pattern>,
}

fn environment_group<'src>() -> impl Parser<'src, &'src str, EnvironmentGroup, E<'src>> {
    pattern()
        .separated_by(just('&').padded_by(inline_whitespace()))
        .collect::<Vec<_>>()
        .map(|patterns| EnvironmentGroup { patterns })
}
#[derive(Debug, Clone, Default)]
pub struct Predicate {
    change: Vec<Change>,
    environment: Vec<EnvironmentGroup>,
    exception: Vec<EnvironmentGroup>,
}

fn environments<'src>() -> impl Parser<'src, &'src str, Vec<EnvironmentGroup>, E<'src>> {
    environment_group()
        .separated_by(just(',').then_ignore(inline_whitespace()))
        .collect::<Vec<_>>()
}

pub fn predicate<'src>() -> impl Parser<'src, &'src str, Predicate, E<'src>> {
    let changes = change()
        .separated_by(just(',').then_ignore(inline_whitespace()))
        .collect::<Vec<_>>();

    let environment_clause = just('/')
        .then(inline_whitespace())
        .ignore_then(environments())
        .or_not()
        .map(|e| e.unwrap_or_default());

    let exception_clause = just('!')
        .then(inline_whitespace())
        .ignore_then(environments())
        .or_not()
        .map(|e| e.unwrap_or_default());

    just('>')
        .ignore_then(inline_whitespace())
        .ignore_then(changes)
        .then_ignore(inline_whitespace())
        .then(environment_clause)
        .then_ignore(inline_whitespace())
        .then(exception_clause) // exceptions
        .map(|((change, environment), exception)| Predicate {
            change,
            environment,
            exception,
        })
}

#[derive(Debug, Clone, Default)]
pub struct Target {
    pattern: Pattern,
    positions: Vec<isize>,
}

#[derive(Debug, Clone, Default)]
pub struct Rule {
    target: Target,
    predicates: Vec<Predicate>,
}

fn rule<'src>() -> impl Parser<'src, &'src str, Rule, E<'src>> {
    let position_num = just('-')
        .or_not()
        .then(digits(10))
        .map_slice(|s| isize::from_str_radix(s, 10))
        .try_map(|t, span| t.map_err(|e| Rich::custom(span, format!("bad number: {e}"))));

    let position = just('@').ignore_then(
        position_num
            .separated_by(just('|'))
            .at_least(1)
            .collect::<Vec<_>>(),
    );

    let target = pattern()
        .then(position.or_not().map(|p| p.unwrap_or_default()))
        .map(|(pattern, positions)| Target { pattern, positions });

    target
        .then_ignore(inline_whitespace())
        .then(
            predicate()
                .separated_by(inline_whitespace().or_not())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .map(|(target, predicates)| Rule { target, predicates })
}

#[derive(Debug, Clone)]
pub enum ASTElement {
    Rule(Rule),
    CatEdit(CategoryEdit),
}

pub fn ast_element<'src>() -> impl Parser<'src, &'src str, ASTElement, E<'src>> {
    choice((
        rule().map(ASTElement::Rule),
        cat_edit().map(ASTElement::CatEdit),
    ))
}

#[derive(Debug, Clone)]
pub struct AST {
    elements: Vec<(ASTElement, SimpleSpan<usize>)>,
}

pub fn ast<'src>() -> impl Parser<'src, &'src str, AST, E<'src>> {
    let comment = just("//")
        .then(any().and_is(newline().not()).repeated())
        .then(newline())
        .padded();
    ast_element()
        .map_with_span(|e, span| (e, span))
        .padded_by(comment.repeated())
        .padded_by(inline_whitespace())
        .separated_by(newline().repeated().at_least(1))
        .collect::<Vec<_>>()
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .map(|elements| AST { elements })
}
