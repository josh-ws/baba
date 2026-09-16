use std::ops::Deref;

use crate::{
    lex::Run,
    unit::{Noun, Operator, Property, Text},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Complement {
    Noun(Noun),
    Property(Property),
}

#[derive(Default)]
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
    Rule {
        subject: Noun::Cursor,
        operator: Operator::Is,
        complement: Complement::Property(Property::Select),
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

#[derive(Debug)]
struct Sentence {
    subjects: Vec<Noun>,
    complements: Vec<(Operator, Complement)>,
}

impl Sentence {
    fn new() -> Self {
        Self {
            subjects: Vec::new(),
            complements: Vec::new(),
        }
    }

    fn rules(&self) -> Vec<Rule> {
        let mut rules = Vec::new();
        for subject in &self.subjects {
            for (op, complement) in &self.complements {
                rules.push(Rule::new(*subject, *op, *complement));
            }
        }
        rules
    }
}

enum State {
    Subject,                  // start, or after AND between subjects
    PostSubject,              // AND, or an operator
    Complement(Operator),     // after an operator
    PostComplement(Operator), // AND, or the sentence is over
    PostAnd(Operator),        // after AND between complements
}

fn sentence(words: &[Text]) -> Option<(Sentence, usize)> {
    let mut sentence = Sentence::new();
    let mut len = 0;
    let mut state = State::Subject;
    for (i, &word) in words.iter().enumerate() {
        state = match (state, word) {
            (State::Subject, Text::Noun(noun)) => {
                sentence.subjects.push(noun);
                State::PostSubject
            }
            (State::PostSubject, Text::And) => State::Subject,
            (State::PostSubject | State::PostAnd(_), Text::Operator(op)) => State::Complement(op),
            (State::Complement(op) | State::PostAnd(op), word) => match complement(op, word) {
                Some(c) => {
                    sentence.complements.push((op, c));
                    len = i + 1;
                    State::PostComplement(op)
                }
                None => break,
            },
            (State::PostComplement(op), Text::And) => State::PostAnd(op),
            _ => break,
        }
    }
    if len > 0 { Some((sentence, len)) } else { None }
}

fn complement(op: Operator, word: Text) -> Option<Complement> {
    match (word, op) {
        (Text::Noun(n), _) => Some(Complement::Noun(n)),
        (Text::Property(p), Operator::Is) => Some(Complement::Property(p)),
        _ => None,
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
        for variant in run.variants() {
            let mut start = 0;
            while start < variant.len() {
                match sentence(&variant[start..]) {
                    Some((sentence, len)) => {
                        for rule in sentence.rules() {
                            rules.add(rule);
                        }
                        start += len - 1;
                    }
                    None => start += 1,
                }
            }
        }
    }
    rules
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
