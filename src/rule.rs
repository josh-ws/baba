use std::ops::Deref;

use crate::{
    lex::Run,
    unit::{Noun, Operator, Property, Text},
};

#[derive(Debug, PartialEq)]
pub enum Complement {
    Noun(Noun),
    Property(Property),
}

pub struct Rules(Vec<Rule>);

const INHERENT: &[Rule] = &[
    Rule {
        subject: Noun::Text,
        operator: Operator::Is,
        complement: Complement::Property(Property::Push),
    },
    Rule {
        subject: Noun::Level,
        operator: Operator::Is,
        complement: Complement::Property(Property::Stop),
    },
];

impl Rules {
    pub fn new() -> Rules {
        Rules(Vec::new())
    }

    pub fn add(&mut self, rule: Rule) {
        self.0.push(rule);
    }

    pub fn unit_has_prop(&self, noun: Noun, property: Property) -> bool {
        self.iter()
            .chain(INHERENT)
            .any(|r| r.subject == noun && r.operator == Operator::Is && r.complement == Complement::Property(property))
    }

    pub fn unit_has(&self, noun: Noun) -> Vec<Noun> {
        self.iter()
            .chain(INHERENT)
            .filter(|r| r.subject == noun && r.operator == Operator::Has)
            .filter_map(|r| match r.complement {
                Complement::Noun(noun) => Some(noun),
                _ => None,
            })
            .collect::<Vec<Noun>>()
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
    fn new(subject: Noun, operator: Operator, complement: Complement) -> Self {
        Self {
            subject,
            operator,
            complement,
        }
    }
}

pub fn parse(runs: &[Run]) -> Rules {
    let mut rules = Rules::new();
    for run in runs {
        for start in 0..run.len() {
            for (rule, _) in rule_at(run, start) {
                rules.add(rule);
            }
        }
    }
    rules
}

fn rule_at(run: &Run, i: usize) -> Vec<(Rule, usize)> {
    let mut out = Vec::new();
    for (subject, j) in noun_at(run, i) {
        for (operator, k) in operator_at(run, j) {
            for (complement, l) in complement_at(run, k, operator) {
                out.push((Rule::new(subject, operator, complement), l));
            }
        }
    }
    out
}

fn noun_at(run: &Run, i: usize) -> Vec<(Noun, usize)> {
    match run.slot(i) {
        Some(t) => t
            .iter()
            .filter_map(|t| match t {
                Text::Noun(n) => Some((*n, i + 1)),
                _ => None,
            })
            .collect(),
        None => Vec::new(),
    }
}

fn operator_at(run: &Run, i: usize) -> Vec<(Operator, usize)> {
    match run.slot(i) {
        Some(t) => t
            .iter()
            .filter_map(|t| match t {
                Text::Operator(n) => Some((*n, i + 1)),
                _ => None,
            })
            .collect(),
        None => Vec::new(),
    }
}

fn complement_at(run: &Run, i: usize, op: Operator) -> Vec<(Complement, usize)> {
    match run.slot(i) {
        Some(t) => t
            .iter()
            .filter_map(|t| match (t, op) {
                (Text::Noun(n), _) => Some((Complement::Noun(*n), i + 1)),
                (Text::Property(p), Operator::Is) => Some((Complement::Property(*p), i + 1)),
                _ => None,
            })
            .collect(),
        None => Vec::new(),
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
        assert_parse_match("BA IS RO", vec!["BA IS RO"]);
    }

    #[test]
    fn parse_multi_rule() {
        assert_parse_match("BA IS RO IS BA", vec!["BA IS RO", "RO IS BA"]);
        assert_parse_match("BA IS RO IS BA IS RO", vec!["BA IS RO", "RO IS BA", "BA IS RO"]);
    }

    #[test]
    fn parse_cross() {
        assert_parse_match(".. BA ..\nRO IS BA\n.. RO ..", vec!["RO IS BA", "BA IS RO"]);
    }

    #[test]
    fn parse_non_square() {
        assert_parse_match("BA IS YO ..\n.. .. .. ..\nRO IS BA ..", vec!["BA IS YO", "RO IS BA"]);
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

    #[test]
    fn parse_has() {
        assert_parse_match("BA HA RO", vec!["BA HA RO"]);
        assert_parse_match("BA HA RO IS YO", vec!["BA HA RO", "RO IS YO"]);
    }

    #[test]
    fn parse_cannot_has_property() {
        assert_parse_match("BA HA YO", vec![]);
    }

    #[test]
    fn parse_ignores_invalid_stacked_rules() {
        assert_parse_match("BA/YO IS RO", vec!["BA IS RO"]);
        assert_parse_match("BA IS/YO RO", vec!["BA IS RO"]);
        assert_parse_match("BA IS RO/YO/IS", vec!["BA IS RO", "BA IS YO"]);
    }

    #[test]
    fn parse_handles_stacked_permutations() {
        assert_parse_match("BA/RO IS YO/WI", vec!["BA IS YO", "BA IS WI", "RO IS YO", "RO IS WI"]);
    }

    #[test]
    fn parse_complement_split() {
        assert_parse_match("BA IS/HA RO", vec!["BA IS RO", "BA HA RO"]);
        assert_parse_match("BA IS/HA YO", vec!["BA IS YO"]);
    }

    #[test]
    fn parse_stacked_cell_mid_run() {
        assert_parse_match("BA IS/RO IS YO", vec!["RO IS YO"]);
    }

    #[test]
    fn parse_two_rules_two_axes() {
        assert_parse_match(
            ".. BA ..\nRO IS/HA BA\n.. RO ..",
            vec!["RO IS BA", "RO HA BA", "BA IS RO", "BA HA RO"],
        );
    }
}
