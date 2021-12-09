use std::iter::Peekable;

use thiserror::Error;

use crate::xpath::{Xpath, XpathElement, XpathPredicate, XpathQuery, tokenizer::{self, Symbol}};

use super::tokenizer::LexError;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Close square bracket has no matching opening square bracket")]
    LeadingCloseBracket,
    #[error("@ symbol cannot be outside of square brackets")]
    MisplacedAtSign,
    #[error("Equals predicate missing assignment sign")]
    PredicateMissingAssignmentSign,
    #[error("Equals predicate missing value")]
    PredicateMissingValue,
    #[error("Equals predicate missing attribute")]
    PredicateMissingAttribute,
    #[error("Lex error {0}")]
    LexError(#[from] LexError)
}

pub fn parse(text: &str) -> Result<Xpath, ParseError> {
    let mut symbols = tokenizer::lex(text)?.into_iter().peekable();
    let mut elements: Vec<XpathElement> = Vec::new();

    while let Some(symbol) = symbols.next() {
        match symbol {
            Symbol::Slash => elements.push(XpathElement::SearchRoot),
            Symbol::DoubleSlash => elements.push(XpathElement::SearchAll),
            Symbol::Identifier(identifier) => {
                let query = parse_query(identifier, &mut symbols)?;
                elements.push(XpathElement::Query(query))
            }
            _ => continue,
        }
    }

    Ok(Xpath { elements })
}

fn parse_query(identifier: String, symbols: &mut Peekable<std::vec::IntoIter<Symbol>>) -> Result<XpathQuery, ParseError> {
    let mut query = XpathQuery::new(identifier);

    let mut open_square_bracket = false;
    while let Some(symbol) = symbols.peek() {
        match symbol {
            Symbol::OpenSquareBracket => {
                symbols.next();
                open_square_bracket = true;
            },
            Symbol::CloseSquareBracket => {
                symbols.next();
                if open_square_bracket {
                    open_square_bracket = false;
                } else {
                    return Err(ParseError::LeadingCloseBracket);
                }
            }
            Symbol::AtSign => {
                symbols.next();
                if !open_square_bracket {
                    return Err(ParseError::MisplacedAtSign);
                }
                let predicate = parse_equals_predicate(symbols)?;
                query.predicates.push(predicate);
            },
            _ => break,
        }
    }

    Ok(query)
}

fn parse_equals_predicate(symbols: &mut Peekable<std::vec::IntoIter<Symbol>>) -> Result<XpathPredicate, ParseError> {
    let mut attr: Option<String> = None;
    let mut val: Option<String> = None;

    println!("{:?}", symbols.peek());
    if let Some(Symbol::Identifier(attribute)) = symbols.next_if(|expected| matches!(expected, &Symbol::Identifier(_))) {
        attr = Some(attribute);
    }

    if let Some(Symbol::AssignmentSign) = symbols.next_if(|expected| matches!(expected, &Symbol::AssignmentSign)) {
        // good
    } else {
        return Err(ParseError::PredicateMissingAssignmentSign);
    }

    if let Some(Symbol::Text(value)) = symbols.next_if(|expected| matches!(expected, &Symbol::Text(_))) {
        val = Some(value);
    }
    
    if let Some(attribute) = attr {
        if let Some(value) = val {
            Ok(XpathPredicate::Equals { attribute, value })
        } else {
            Err(ParseError::PredicateMissingValue)
        }
    } else {
        Err(ParseError::PredicateMissingAttribute)
    }
}

#[cfg(test)]
mod tests {
    use crate::xpath::XpathPredicate;

    use super::*;

    #[test]
    fn parse_works() {
        let text = "//book/title";

        let result = parse(text).unwrap();

        let expected = vec![
            XpathElement::SearchAll,
            XpathElement::Query(XpathQuery::new(String::from("book"))),
            XpathElement::SearchRoot,
            XpathElement::Query(XpathQuery::new(String::from("title")))
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result.elements) {
            assert_eq!(e, r);
        }
    }

    #[test]
    fn parse_attribute() {
        let text = r###"//a[@hello="world"]"###;

        let result = parse(text).unwrap();

        let expected = vec![
            XpathElement::SearchAll,
            XpathElement::Query(
                XpathQuery {
                    identifier: String::from("a"),
                    predicates: vec![
                        XpathPredicate::Equals {
                            attribute: String::from("hello"),
                            value: String::from("world")
                        }
                    ]
                }
            ),
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result.elements) {
            assert_eq!(e, r);
        }
    }
}
