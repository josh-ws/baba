use std::collections::HashMap;

use crate::level::Level;

#[derive(Debug)]
pub struct Levelpack {
    name: String,
    entry_point: String,
    levels: HashMap<String, Level>,
}

impl Levelpack {
    pub fn parse(src: &str) -> Result<Self, String> {
        let mut pack = Levelpack::blank();
        let mut key = String::new();
        let mut body = String::new();
        for line in src.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[") && trimmed.ends_with("]") {
                pack.add_key_value(&key, &body)?;
                key = trimmed.trim_matches(['[', ']']).to_string();
                body.clear();
            } else {
                body.push_str(line);
                body.push('\n');
            }
        }
        pack.add_key_value(&key, &body)?;
        pack.validate_links()?;
        if !pack.levels.contains_key(pack.root()) {
            return Err(format!("invalid levelpack root: `{}`", pack.root()));
        }
        Ok(pack)
    }

    fn validate_links(&self) -> Result<(), String> {
        let mut unresolved = self
            .levels
            .iter()
            .flat_map(|(key, level)| {
                level
                    .links()
                    .values()
                    .filter(|link_key| !self.levels.contains_key(*link_key))
                    .map(move |link_key| (key.as_str(), link_key.as_str()))
            })
            .collect::<Vec<(&str, &str)>>();

        if unresolved.is_empty() {
            return Ok(());
        }
        unresolved.sort_unstable();
        unresolved.dedup();
        let detail = unresolved
            .iter()
            .map(|(level, link)| format!("'{level}' -> '{link}'"))
            .collect::<Vec<String>>()
            .join(", ");
        Err(format!("levelpack has invalid links: {detail}"))
    }

    pub fn has_level(&self, name: &str) -> bool {
        self.levels.contains_key(name)
    }

    pub fn root(&self) -> &str {
        &self.entry_point
    }

    pub fn get_level(&self, key: &str) -> Option<&Level> {
        self.levels.get(key)
    }

    pub fn get_level_mut(&mut self, key: &str) -> Option<&mut Level> {
        self.levels.get_mut(key)
    }

    fn add_key_value(&mut self, key: &str, body: &str) -> Result<(), String> {
        if key.is_empty() {
            for (k, v) in Levelpack::fields(body) {
                match k {
                    "Pack" => self.name = v.to_string(),
                    "Root" => self.entry_point = v.to_string(),
                    _ => return Err(format!("unrecognised pack key {k}")),
                }
            }
        } else {
            let level = Level::read(body).map_err(|e| format!("level {key}: {e}"))?;
            if self.levels.contains_key(key) {
                return Err(format!("levelpack has duplicate level key {key}"));
            }
            self.levels.insert(key.to_string(), level);
        }
        Ok(())
    }

    fn fields(src: &str) -> Vec<(&str, &str)> {
        src.lines()
            .filter_map(|l| l.split_once('='))
            .map(|(k, v)| (k.trim(), v.trim()))
            .collect::<Vec<(&str, &str)>>()
    }

    fn blank() -> Self {
        Self {
            name: String::new(),
            entry_point: String::new(),
            levels: HashMap::new(),
        }
    }
}
