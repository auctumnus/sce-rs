use chumsky::{
    prelude::*,
    text::{digits, inline_whitespace, newline, whitespace},
};
use std::str::FromStr;

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
    pub target: String,
    pub elements: Vec<CatOrEl>,
    pub kind: CategoryEditKind,
}

fn escape<'a>() -> impl Parser<'a, &'a str, char, E<'a>> {
    just('\\').ignore_then(one_of(CONTROL_CHARACTERS))
}
#[cfg(test)]
mod escape_tests {
    use chumsky::Parser;

    #[test]
    fn basic() {
        assert_eq!(
            crate::parse::escape().parse("\\[").into_output_errors(),
            (Some('['), vec![])
        );
    }
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
mod text_tests {
    use chumsky::Parser;
    #[test]
    fn basic() {
        let passing_cases = [("abc", "abc"), ("\\[a\\]", "[a]")];

        for (input, expected) in passing_cases {
            let (parsed, errs) = crate::parse::text().parse(input).into_output_errors();
            assert_eq!(parsed, Some(String::from(expected)));
            assert!(errs.is_empty());
        }

        let failing_cases = ["\\", "\\n", "a "];

        for input in failing_cases {
            let (parsed, errs) = crate::parse::text().parse(input).into_output_errors();
            assert!(parsed.is_none());
            assert!(!errs.is_empty());
        }
    }
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
mod cat_or_els_tests {
    use chumsky::Parser;
    #[test]
    fn cat_or_els_test() {
        use super::CatOrEl::*;
        assert_eq!(
            super::cat_or_els().parse("a,b,[c]").into_output(),
            Some(vec![
                El(String::from("a")),
                El(String::from("b")),
                Cat(String::from("c"))
            ])
        );
    }
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
            elements,
            kind,
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
        .try_map(|t: &str, span| {
            t.parse::<usize>()
                .map_err(|_| Rich::custom(span, "couldn't parse repeat int"))
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
    pub elements: Vec<PatternElement>,
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
mod pattern_tests {
    use chumsky::Parser;
    #[test]
    fn basic() {
        use super::PatternElement::*;
        use super::Wildcard::*;

        let cases = [
            ("a", vec![Text(String::from("a"))]),
            ("*", vec![Wildcard(Greedy)]),
        ];

        for (input, expected) in cases {
            let actual = super::pattern()
                .parse(input)
                .into_output()
                .map(|p| p.elements);
            assert_eq!(actual, Some(expected));
        }
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

fn environment_clause<'src>() -> impl Parser<'src, &'src str, Vec<EnvironmentGroup>, E<'src>> {
    just('/')
        .then(inline_whitespace())
        .ignore_then(environments())
        .or_not()
        .map(|e| e.unwrap_or_default())
}

fn exception_clause<'src>() -> impl Parser<'src, &'src str, Vec<EnvironmentGroup>, E<'src>> {
    just('!')
        .then(inline_whitespace())
        .ignore_then(environments())
        .or_not()
        .map(|e| e.unwrap_or_default())
}

pub fn predicate<'src>() -> impl Parser<'src, &'src str, Predicate, E<'src>> {
    let changes = change()
        .separated_by(just(',').then_ignore(inline_whitespace()))
        .collect::<Vec<_>>();

    just('>')
        .ignore_then(inline_whitespace())
        .ignore_then(changes)
        .then_ignore(inline_whitespace())
        .then(environment_clause())
        .then_ignore(inline_whitespace())
        .then(exception_clause())
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

fn predicates<'src>() -> impl Parser<'src, &'src str, Vec<Predicate>, E<'src>> {
    predicate()
        .separated_by(inline_whitespace().or_not())
        .at_least(1)
        .collect::<Vec<_>>()
}

fn target<'src>() -> impl Parser<'src, &'src str, Target, E<'src>> {
    let position_num = just('-')
        .or_not()
        .then(digits(10))
        .map_slice(str::parse)
        .try_map(|t, span| t.map_err(|e| Rich::custom(span, format!("bad number: {e}"))));

    let position = just('@').ignore_then(
        position_num
            .separated_by(just('|'))
            .at_least(1)
            .collect::<Vec<_>>(),
    );

    pattern()
        .then(position.or_not().map(Option::unwrap_or_default))
        .map(|(pattern, positions)| Target { pattern, positions })
}

fn rule<'src>() -> impl Parser<'src, &'src str, Rule, E<'src>> {
    let rule = target()
        .then_ignore(inline_whitespace())
        .then(predicates())
        .map(|(target, predicates)| Rule { target, predicates });

    // yes, epenthesis can just have an arbitrary predicate. no, i have no clue why
    // see: application of `+ a > b / c` to words `ac`, `ab` results in `aaaca`, `aaaba`
    let epenthesis = just('+')
        .ignore_then(target().padded_by(inline_whitespace()))
        .then(predicates())
        .map(|(target, mut predicates)| {
            // set the target to null, and move the target to the change
            // such that `+ a / _b` == `[] > a / _b`

            let null_target = Target {
                pattern: Pattern {
                    elements: vec![PatternElement::Category(vec![])],
                },
                positions: target.positions,
            };

            predicates[0].change = vec![Change {
                pattern: target.pattern,
            }];

            Rule {
                target: null_target,
                predicates,
            }
        });

    let deletion = just('-')
        .ignore_then(target().padded_by(inline_whitespace()))
        .then(predicates())
        .map(|(target, predicates)| {
            // set change to null such that `- a / _b` == `a > [] / _b`

            let predicates = predicates
                .into_iter()
                .map(|predicate| {
                    let null_change = vec![Change {
                        pattern: Pattern {
                            elements: vec![PatternElement::Category(vec![])],
                        },
                    }];
                    Predicate {
                        change: null_change,
                        environment: predicate.environment,
                        exception: predicate.exception,
                    }
                })
                .collect();

            Rule { target, predicates }
        });

    choice((rule, epenthesis, deletion))
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

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub struct AST {
    pub elements: Vec<(ASTElement, SimpleSpan<usize>)>,
}
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

#[cfg(test)]
mod bench {
    extern crate test;
    use chumsky::Parser;
    use test::Bencher;

    #[bench]
    fn ast_bench(b: &mut Bencher) {
        // saxonish sound changes
        // https://conworkshop.com/view_language.php?l=sxs
        // shoutout!
        let input = r#"
    N=m,n
    T=p,t,k
    D=b,d,g
    F=f,þ,s,z,h
    R=w,r,l,j
    C=[N],[T],[D],[F],[R]
    
    V=i,u,ī,ū,e,ē,ê,ō,ô,a,ā,ą,į,ų,į̄,ǭ,ǫ̂
    
    // west germanic
    i, u > e, o / _[C]{*}[a,ā,ą] ! _[n,j], _[C]{*}[n,j], a_
    ē > æ: ! _#
    a, o, u > æ, e, i / _[C]{*}[i,j] ! _i
    u > o / _[C]{*}[a,ā,ą] ! _[n,j], _[C]{*}[n,j]
    ō, ē, ǭ > u, a, ā / _#
    ai, au > æ:, ā
    - z / _#
    a,ą > ă / _#
    zw,dw > ww
    z > r
    j > "j / [C]_ ! r_
    
    // ingvaeonic
    a[N], e[N], i[N], ō[N], u[N], ī[N], ū[N], ē[N], ā[N] > ą, ę, į, ǭ, ų, į̄, ų̄, ę̄, ą̄ / _[F]
    a > æ ! _[N], _[C]{*}[a,ā,ą,ą̄,ō,ǭ,u,ų,ų̄]
    
    // ortho convert
    ī, ē, ā, ō, ū > i:, e:, ɑ:, o:, u:
    ǭ, į̄, ų̄, ę̄, ą̄ > ǫ:, į:, ų:, ę:, ą:
    V += æ, ą, ę, į, ǫ, ų
    
    // old saxonish
    m,b,d,g > w̃,w,ð,ɣ / [V](:)_[[V],ă]
    p, t, k > f, þ, h / [V](:)_[[V],ă]
    - ă
    C += w̃,ð,ɣ
    [N][T],[N][D] > [N][D], [N][N]
    sk > sʲ
    + ʲ / [#,[C]]_[i,j,e,į,ę,æ], [i,į](:)[C]_[#,[C]]
    + ʲ / [C]_[C]ʲ, ʲ[C]_ ! ʲ_
    ɣʲ > ʝ
    a, e, o > ɔ, i, u / _[N]
    ą, ę, ǫ > ɔ, i, u
    ą, ę, į, ǫ, ų > a, e, i, o, u
"#;
        b.iter(|| crate::parse::ast().parse(input).into_output_errors());
    }
}
