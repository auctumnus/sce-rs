use std::fmt::Display;

use crate::parse::Pattern;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Word {
    pub phones: Vec<String>,
    pub graphs: Vec<String>,
    pub separator: String,
}

impl Word {
    /// Match a pattern to the phonemes of a word, starting from the given index.
    ///
    /// ## Returns
    /// TODO
    fn match_one(&self, pattern: &Pattern, start_index: usize) -> Option<()> {
        use crate::parse::PatternElement::*;

        let mut index = start_index;
        let mut last_index = start_index;

        // TODO: could be more rusty

        for element_index in 0..pattern.elements.len() {
            let element = &pattern.elements[element_index];
            let phone = &self.phones[element_index];
            match element {
                Text(text) => {
                    // TODO: this is technically incorrect, since the parser
                    // just swallows text without thinking about graphs
                    // i think i need an intermediate step to convert these??
                    if text != phone {
                        return None;
                    }
                }
                Ditto => {
                    if element_index == 0 || phone != &self.phones[element_index - 1] {
                        return None;
                    }
                }
                _ => todo!(),
            }
        }

        None
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if no_polygraphs(&self.graphs) {
            let mut as_str = String::new();
            for phone in &self.phones {
                if phone == "#" {
                    as_str.push(' ');
                } else {
                    as_str.push_str(phone.as_str());
                }
            }
            write!(f, "{}", as_str.trim())
        } else {
            let mut as_str = String::new();
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
    true
}

pub fn into_phones(input: String, graphs: &Vec<String>, separator: &String) -> Vec<String> {
    let mut phones: Vec<String> = vec![];
    let mut input = input;

    while !input.is_empty() {
        if input.starts_with(separator) {
            input = input.split_once(separator).unwrap().1.to_string();
        }

        let graph = graphs.iter().find(|g| input.starts_with(g.as_str()));

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

    phones
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

    graphs.sort_by_cached_key(String::len);
    graphs.reverse();

    if no_polygraphs(&graphs) {
        let phones = input
            .split("")
            .filter(|s| !s.is_empty() && s != &separator.as_str())
            .map(ToString::to_string)
            .collect();
        return Word {
            phones,
            graphs,
            separator,
        };
    }

    let phones = into_phones(input, &graphs, &separator);

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

        assert_eq!(word.to_string(), "abc".to_string());
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
