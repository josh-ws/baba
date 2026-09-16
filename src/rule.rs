use std::ops::Deref;

use crate::unit::{Noun, Operator, Property};

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
        if !self.0.contains(&rule) {
            self.0.push(rule);
        }
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
    pub fn new(subject: Noun, operator: Operator, complement: Complement) -> Self {
        Self {
            subject,
            operator,
            complement,
        }
    }
}
