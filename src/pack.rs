use std::collections::HashMap;

use crate::level::Level;

#[derive(Debug, Default)]
pub struct Levelpack {
    name: String,
    entry_point: String,
    levels: HashMap<String, Level>,
}

impl Levelpack {
    pub fn parse(src: &str) -> Self {
        let mut pack = Levelpack::default();
        let mut key = String::new();
        let mut body = String::new();
        for line in src.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[") && trimmed.ends_with("]") {
                pack.add_key_value(&key, &body);
                key = trimmed.trim_matches(['[', ']']).to_string();
                body.clear();
            } else {
                body.push_str(line);
                body.push('\n');
            }
        }
        pack.add_key_value(&key, &body);
        pack
    }

    pub fn root(&self) -> &str {
        &self.entry_point
    }

    pub fn get_level(&self, key: &str) -> &Level {
        self.levels.get(key).expect("level {key} does not exist")
    }

    pub fn get_level_mut(&mut self, key: &str) -> &mut Level {
        self.levels.get_mut(key).expect("level {key} does not exist")
    }

    fn add_key_value(&mut self, key: &str, body: &str) {
        if key.is_empty() {
            for (k, v) in Levelpack::fields(body) {
                match k {
                    "Pack" => self.name = v.to_string(),
                    "Root" => self.entry_point = v.to_string(),
                    _ => panic!("unrecognised pack key {k}"),
                }
            }
        } else {
            self.levels.insert(key.to_string(), Level::read(body));
        }
    }

    fn fields(src: &str) -> Vec<(&str, &str)> {
        src.lines()
            .filter_map(|l| l.split_once('='))
            .map(|(k, v)| (k.trim(), v.trim()))
            .collect::<Vec<(&str, &str)>>()
    }
}
