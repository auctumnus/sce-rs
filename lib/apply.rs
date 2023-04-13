use std::collections::HashMap;

use crate::{
    parse::{ASTElement, CatOrEl, CategoryEditKind, AST},
    word::into_phones,
};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub elements: Vec<Vec<String>>,
}

#[derive(Default, Debug)]
pub struct InterpreterState {
    pub graphs: Vec<String>,
    pub categories: HashMap<String, Category>,
}

fn without<T: PartialEq>(input: Vec<T>, items: Vec<T>) -> Vec<T> {
    let mut new_input = vec![];
    for item in input {
        if !items.contains(&item) {
            new_input.push(item);
        }
    }
    new_input
}

fn cat_or_els_to_els(
    elements: Vec<CatOrEl>,
    categories: &HashMap<String, Category>,
    graphs: &Vec<String>,
    separator: &String,
) -> Vec<Vec<String>> {
    use CatOrEl::*;
    let mut new_elements = vec![];

    for e in elements {
        match e {
            Cat(name) => {
                if let Some(category) = categories.get(&name) {
                    let mut cat_elements = category.elements.clone();
                    new_elements.append(&mut cat_elements);
                }
            }
            El(input) => new_elements.push(into_phones(input, graphs, separator)),
        }
    }

    new_elements
}

/// Applies the rules found in the given syntax tree to a set of words,
/// parsing the words using the given graphs and separator.
///
/// ## Returns
/// The transformed words.
pub fn apply(
    ast: AST,
    words: Vec<String>,
    graphs: Vec<String>,
    separator: String,
) -> Result<(Vec<String>, InterpreterState), ()> {
    let parsed_words: Vec<_> = words
        .iter()
        .map(|word| crate::word::parse(word, graphs.clone(), separator.clone()))
        .collect();

    let state = ast.elements.into_iter().map(|(element, _)| element).fold(
        InterpreterState::default(),
        |mut state, element| {
            use ASTElement::*;
            use CategoryEditKind::*;
            println!("{state:?}");
            match element {
                Rule(rule) => state,
                CatEdit(edit) => {
                    let name = edit.target;
                    let mut elements =
                        cat_or_els_to_els(edit.elements, &state.categories, &graphs, &separator);
                    match edit.kind {
                        Def => {
                            let category = Category { elements };

                            state.categories.insert(name, category);
                        }
                        Add => {
                            if let Some(category) = state.categories.get(&name) {
                                let mut category = category.clone();
                                category.elements.append(&mut elements);
                                state.categories.insert(name, category);
                            }
                        }
                        Sub => {
                            if let Some(category) = state.categories.get(&name) {
                                let mut category = category.clone();

                                category.elements = without(category.elements, elements);

                                state.categories.insert(name, category);
                            }
                        }
                    };

                    state
                }
            }
        },
    );

    Ok((vec![], state))
}

#[cfg(test)]
mod apply_tests {
    use std::collections::HashMap;

    use super::{apply, Category};
    use crate::parse::ast;
    use chumsky::Parser;
    #[test]
    fn cat_basic() {
        let ast = ast().parse("A = b,c,d").into_output().unwrap();
        let (_, state) = apply(ast, vec!["a".to_string()], vec![], "'".to_string()).unwrap();

        assert_eq!(
            state.categories.get("A"),
            Some(&Category {
                elements: vec![
                    vec!["b".to_string()],
                    vec!["c".to_string()],
                    vec!["d".to_string()]
                ]
            })
        );
    }
}
