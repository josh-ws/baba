use crate::{
    lex::Run,
    rule::{Complement, Rule, Rules},
    unit::{Noun, Operator, Text},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Subject,
    AfterSubject,
    SubjectAnd,
    Complement(Operator),
    AfterComplement(Operator, Option<Noun>),
    AfterAnd(Operator),
}

#[derive(Clone, Debug)]
struct Parser {
    state: State,
    subjects: Vec<Noun>,
}

impl Parser {
    fn new(state: State, subjects: Vec<Noun>) -> Self {
        Self { state, subjects }
    }

    fn start(word: Text) -> Self {
        match word {
            Text::Noun(n) => Self {
                state: State::AfterSubject,
                subjects: vec![n],
            },
            _ => Self {
                state: State::Subject,
                subjects: Vec::new(),
            },
        }
    }

    fn with(&self, state: State) -> Self {
        Self {
            state,
            subjects: self.subjects.clone(),
        }
    }

    fn merge(&mut self, other: Parser) {
        for noun in other.subjects {
            self.add_subject(noun);
        }
    }

    fn add_subject(&mut self, noun: Noun) {
        if !self.subjects.contains(&noun) {
            self.subjects.push(noun);
        }
    }

    fn step(&self, word: Text, rules: &mut Rules) -> Self {
        match (self.state, word) {
            (State::Subject, _) => Self::start(word),
            (State::AfterSubject, Text::Operator(o)) => self.with(State::Complement(o)),
            (State::AfterSubject, Text::And) => self.with(State::SubjectAnd),
            (State::SubjectAnd, Text::Noun(n)) => {
                let mut next = self.with(State::AfterSubject);
                next.add_subject(n);
                next
            }
            (State::AfterComplement(o, _), Text::And) => self.with(State::AfterAnd(o)),
            (State::AfterComplement(_, Some(n)), Text::Operator(o)) => Self {
                state: State::Complement(o),
                subjects: vec![n],
            },
            (State::AfterAnd(_), Text::Operator(o)) => self.with(State::Complement(o)),
            (State::Complement(o) | State::AfterAnd(o), _) => match complement(o, word) {
                Some(c) => {
                    for &n in &self.subjects {
                        rules.add(Rule::new(n, o, c));
                    }
                    let last = match c {
                        Complement::Noun(n) => Some(n),
                        Complement::Property(_) => None,
                    };
                    self.with(State::AfterComplement(o, last))
                }
                None => Self::start(word),
            },
            _ => Self::start(word),
        }
    }
}

fn complement(op: Operator, word: Text) -> Option<Complement> {
    match (word, op) {
        (Text::Noun(n), _) => Some(Complement::Noun(n)),
        (Text::Property(p), Operator::Is) => Some(Complement::Property(p)),
        _ => None,
    }
}

pub fn parse(runs: &[Run]) -> Rules {
    let mut rules = Rules::new();
    for run in runs {
        let mut live = vec![Parser::new(State::Subject, vec![])];
        for slot in run.slots() {
            let mut next: Vec<Parser> = Vec::new();
            for parser in &live {
                for &word in slot {
                    let stepped = parser.step(word, &mut rules);
                    match next.iter_mut().find(|p| p.state == stepped.state) {
                        Some(existing) => existing.merge(stepped),
                        None => next.push(stepped),
                    }
                }
            }
            live = next;
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
        assert_parse_match("BA IS RO IS BA IS RO", vec!["BA IS RO", "RO IS BA"]);
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
        assert_parse_match("BA/RO IS YO/WI", vec!["BA IS YO", "RO IS YO", "BA IS WI", "RO IS WI"]);
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

    #[test]
    fn parse_and() {
        assert_parse_match("BA AN RO IS YO", vec!["BA IS YO", "RO IS YO"]);
        assert_parse_match("BA IS YO AN WI", vec!["BA IS YO", "BA IS WI"]);
        assert_parse_match(
            "BA AN RO IS YO AN WI",
            vec!["BA IS YO", "RO IS YO", "BA IS WI", "RO IS WI"],
        );
        assert_parse_match("BA IS RO AN HA KE AN IS WA", vec!["BA IS RO", "BA HA KE", "BA IS WA"]);
        assert_parse_match("BA HA RO AN KE", vec!["BA HA RO", "BA HA KE"]);
    }
}
