use std::ops::Deref;

use crate::{
    lex::Run,
    unit::{Noun, Operator, Property, Text},
};

#[derive(Debug, PartialEq)]
pub enum Complement {
    Transformation(Noun),
    Property(Property),
}

pub struct Rules(Vec<Rule>);

impl Rules {
    pub fn new() -> Rules {
        Rules(Vec::new())
    }

    pub fn add(&mut self, rule: Rule) {
        self.0.push(rule);
    }

    pub fn has(&self, noun: Noun, property: Property) -> bool {
        self.iter().any(|r| {
            r.subject == noun
                && r.operator == Operator::Is
                && r.complement == Complement::Property(property)
        })
    }
}

impl Deref for Rules {
    type Target = [Rule];

    fn deref(&self) -> &[Rule] {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub subject: Noun,
    pub operator: Operator,
    pub complement: Complement,
}

impl Rule {
    pub fn new(subject: Noun, operator: Operator, complement: Complement) -> Self {
        Self {
            subject,
            operator,
            complement,
        }
    }

    pub fn from_transformation(target: Noun, transform_to: Noun) -> Self {
        Self {
            subject: target,
            operator: Operator::Is,
            complement: Complement::Transformation(transform_to),
        }
    }
}

pub fn parse(runs: &[Run]) -> Rules {
    let mut rules = Rules::new();
    for run in runs {
        for window in run.words().windows(3) {
            if let Some(rule) = parse_rule(window) {
                rules.add(rule);
            }
        }
    }
    rules
}

fn parse_rule(window: &[Text]) -> Option<Rule> {
    Some(Rule {
        subject: noun(window[0])?,
        operator: operator(window[1])?,
        complement: complement(window[2])?,
    })
}

fn noun(t: Text) -> Option<Noun> {
    match t {
        Text::Noun(n) => Some(n),
        _ => None,
    }
}

fn operator(t: Text) -> Option<Operator> {
    match t {
        Text::Operator(o) => Some(o),
        _ => None,
    }
}

fn complement(t: Text) -> Option<Complement> {
    match t {
        Text::Noun(n) => Some(Complement::Transformation(n)),
        Text::Property(n) => Some(Complement::Property(n)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use crate::{lex::lex, world::Grid};

    use super::*;

    #[track_caller]
    fn assert_parse_match(src: &str, exp: Vec<&str>) {
        let got = parse(&lex(&Grid::from_ascii(src)))
            .iter()
            .map(Rule::to_ascii)
            .collect::<Vec<String>>();
        let exp = exp.into_iter().map(str::to_string).collect::<Vec<String>>();
        assert_eq!(got, exp);
    }

    #[test]
    fn parse_finds_rules() {
        assert_parse_match("BA IS YO", vec!["BA IS YO"]);
        assert_parse_match("BA IS KE", vec!["BA IS KE"]);
    }

    #[test]
    fn parse_multi_rule() {
        assert_parse_match("BA IS KE IS BA", vec!["BA IS KE", "KE IS BA"]);
        assert_parse_match(
            "BA IS KE IS BA IS KE",
            vec!["BA IS KE", "KE IS BA", "BA IS KE"],
        );
    }

    #[test]
    fn parse_cross() {
        assert_parse_match(".. BA ..\nKE IS BA\n.. KE ..", vec!["KE IS BA", "BA IS KE"]);
    }

    #[test]
    fn parse_non_square() {
        assert_parse_match(
            "BA IS YO ..\n.. .. .. ..\nKE IS BA ..",
            vec!["BA IS YO", "KE IS BA"],
        );
    }

    #[test]
    fn parse_rule_breaks() {
        assert_parse_match("BA IS .. YO", vec![]);
        assert_parse_match("BA IS ba YO", vec![]);
    }

    #[test]
    fn parse_rejects_non_rules() {
        assert_parse_match("IS IS IS", vec![]);
        assert_parse_match("BA YO IS", vec![]);
        assert_parse_match("YO IS BA", vec![]);
        assert_parse_match("BA IS", vec![]);
    }

    #[test]
    fn parse_finds_nothing() {
        assert_parse_match("", vec![]);
        assert_parse_match(".. .. ..", vec![]);
        assert_parse_match("ba ba ba", vec![]);
    }
}
