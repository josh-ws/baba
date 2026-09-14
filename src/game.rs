use crate::{
    eval::{Event, TurnStatus},
    level::Level,
    lex::lex,
    pack::Levelpack,
    query::query_selected,
    rule::parse,
    world::Direction,
};

#[derive(Debug)]
pub struct Game {
    pack: Levelpack,
    selected: Vec<u64>,
    visited: Vec<String>,
}

impl Game {
    pub fn new(pack: Levelpack) -> Self {
        let root = pack.root().to_string();
        let mut game = Self {
            pack,
            selected: Vec::new(),
            visited: vec![root],
        };
        game.update_selected();
        game
    }

    pub fn from_file(pack_path: &str) -> Result<Self, String> {
        let pack = Game::read_pack(pack_path)?;
        Ok(Self::new(pack))
    }

    pub fn reload(&mut self, path: &str) -> Result<(), String> {
        let root = self.visited.first().expect("visited cannot be empty");
        self.pack = Game::read_pack(path)?;

        // new pack does not have the root. construct a new root
        if !self.pack.has_level(root) {
            let root = &self.pack.root().to_string();
            self.visited.clear();
            self.goto(root);
        }

        // new pack has removed, or renamed, levels in our path
        if let Some(i) = self.visited.iter().position(|k| !self.pack.has_level(k)) {
            self.visited.truncate(i);
        }

        self.update_selected();
        Ok(())
    }

    pub fn undo(&mut self) {
        self.current_level_mut().undo();
        self.update_selected();
    }

    pub fn current_level(&self) -> &Level {
        let current = self.visited.last().expect("empty visited");
        self.pack.get_level(current).expect("current_level is checked")
    }

    pub fn current_level_mut(&mut self) -> &mut Level {
        let current = self.visited.last().expect("empty visited");
        self.pack.get_level_mut(current).expect("current_level is checked")
    }

    pub fn current_key(&self) -> &str {
        self.visited.last().expect("empty visited")
    }

    pub fn update(&mut self, dir: Direction) -> Vec<Event> {
        let result = self.current_level_mut().update(dir);
        self.selected = result.selected;
        if result.status == TurnStatus::Win {
            println!("You win!");
            self.return_to_parent();
        }
        result.events
    }

    pub fn caption(&self) -> Option<&str> {
        let key = self.current_level().link_for(&self.selected);
        match key {
            Some(key) => match self.pack.get_level(key) {
                Some(level) => Some(level.name()),
                None => None,
            },
            None => None,
        }
    }

    pub fn enter_link(&mut self) -> bool {
        let Some(link) = self.current_level().link_for(&self.selected) else {
            return false;
        };
        let link = link.to_string();
        self.goto(&link);
        true
    }

    pub fn return_to_parent(&mut self) {
        if self.visited.len() > 1 {
            self.visited.pop();
            self.update_selected();
        }
    }

    fn goto(&mut self, key: &str) {
        debug_assert!(self.pack.get_level(key).is_some());
        self.visited.push(key.to_string());
        self.update_selected();
    }

    fn update_selected(&mut self) {
        let rules = parse(&lex(self.current_level().grid()));
        let selected = query_selected(&rules, self.current_level().grid());
        self.selected = selected.iter().map(|u| u.unit_id).collect();
    }

    fn read_pack(path: &str) -> Result<Levelpack, String> {
        match std::fs::read_to_string(path) {
            Ok(src) => Ok(Levelpack::parse(&src)?),
            Err(e) => Err(format!("could not load levelpack from {}: {}", path, e)),
        }
    }
}
