use std::fmt::Display;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Word {
    phones: Vec<String>,
    graphs: Vec<String>,
    separator: String,
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if no_polygraphs(&self.graphs) {
            let mut as_str = String::from("");
            for phone in &self.phones {
                if phone == "#" {
                    as_str.push(' ');
                } else {
                    as_str.push_str(phone.as_str())
                }
            }
            write!(f, "{}", as_str.trim())
        } else {
            let mut as_str = String::from("");
            for phone in &self.phones {
                if phone == "#" {
                    as_str.push(' ');
                } else {
                    as_str.push_str(phone.as_str());
                    if self
                        .graphs
                        .iter()
                        .any(|graph| graph.starts_with(phone) && graph != phone)
                    {
                        as_str.push_str(&self.separator);
                    }
                }
            }
            write!(f, "{}", as_str.trim())
        }
    }
}

fn no_polygraphs(graphs: &Vec<String>) -> bool {
    for graph in graphs {
        if graph.len() > 1 {
            return false;
        }
    }
    return true;
}

/// Parses an input string into a word.
/// Takes ownership of the `graphs` in order to preserve which ones were the ones
/// used to parse the word.
///
/// ## Returns
///
/// The resultant `Word`.
pub fn parse(input: &String, mut graphs: Vec<String>, separator: String) -> Word {
    let input = input.split_whitespace().collect::<Vec<_>>().join("#");
    let input = format!("#{input}#");

    graphs.sort_by_cached_key(|g| g.len());
    graphs.reverse();

    if no_polygraphs(&graphs) {
        let phones = input
            .split("")
            .filter(|s| s.len() != 0 && s != &separator.as_str())
            .map(|s| s.to_string())
            .collect();
        return Word {
            phones,
            graphs,
            separator,
        };
    }

    let mut phones: Vec<String> = vec![];
    let mut input = input;

    while input.len() > 0 {
        if input.starts_with(&separator) {
            input = input.split_once(&separator).unwrap().1.to_string();
        }

        let graph = graphs
            .iter()
            .filter(|g| input.starts_with(g.as_str()))
            .next();

        if let Some(graph) = graph {
            let len = graph.len();
            phones.push(graph.to_string());
            input = input[len..].to_string();
        } else {
            let first = input.split_at(1).0;
            phones.push(first.to_string());
            input = input[1..].to_string();
        }
    }

    Word {
        phones,
        graphs,
        separator,
    }
}

#[cfg(test)]
mod word_tests {
    use super::parse;

    #[test]
    fn basic() {
        let input = String::from("abc");
        let graphs = vec![];
        let separator = String::from("'");

        let word = parse(&input, graphs, separator);

        assert_eq!(
            word.phones,
            vec![
                String::from("#"),
                String::from("a"),
                String::from("b"),
                String::from("c"),
                String::from("#")
            ]
        );
    }

    #[cfg(test)]
    #[test]
    fn unnecessary_separator() {
        let input = String::from("a'bc");
        let graphs = vec![];
        let separator = String::from("'");

        let word = parse(&input, graphs, separator);

        assert_eq!(
            word.phones,
            vec![
                String::from("#"),
                String::from("a"),
                String::from("b"),
                String::from("c"),
                String::from("#")
            ]
        );

        assert_eq!(word.to_string(), "abc".to_string())
    }

    #[cfg(test)]
    #[test]
    fn polygraphs() {
        let input = "atshu".into();
        let graphs = vec!["sh".into(), "ts".into(), "tsh".into()];
        let separator = String::from("'");

        let word = parse(&input, graphs, separator);

        assert_eq!(
            word.phones,
            vec![
                String::from("#"),
                String::from("a"),
                String::from("tsh"),
                String::from("u"),
                String::from("#")
            ]
        );

        assert_eq!(word.to_string(), input);

        let input = "ats'hu".into();
        let graphs = vec!["sh".into(), "ts".into(), "tsh".into()];
        let separator = String::from("'");

        let word = parse(&input, graphs, separator);

        assert_eq!(
            word.phones,
            vec![
                String::from("#"),
                String::from("a"),
                String::from("ts"),
                String::from("h"),
                String::from("u"),
                String::from("#")
            ]
        );

        assert_eq!(word.to_string(), input);
    }

    #[cfg(test)]
    #[test]
    fn internal_whitespace() {
        let input = "a  b".into();

        let word = parse(&input, vec![], String::from("'"));

        assert_eq!(
            word.phones,
            vec![
                String::from("#"),
                String::from("a"),
                String::from("#"),
                String::from("b"),
                String::from("#")
            ]
        );
    }
}
