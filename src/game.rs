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
    current_level: String,
    selected: Vec<u64>,
}

impl Game {
    pub fn new(pack: Levelpack) -> Self {
        let mut game = Self {
            current_level: pack.root().to_string(),
            pack,
            selected: Vec::new(),
        };
        game.update_selected();
        game
    }

    pub fn from_file(pack_path: &str) -> Result<Self, String> {
        let pack = Game::read_pack(pack_path)?;
        Ok(Self::new(pack))
    }

    pub fn reload(&mut self, path: &str) -> Result<(), String> {
        self.pack = Game::read_pack(path)?;
        if !self.pack.has_level(&self.current_level) {
            self.return_to_root();
        } else {
            self.update_selected();
        }
        Ok(())
    }

    pub fn undo(&mut self) {
        self.current_level_mut().undo();
        self.update_selected();
    }

    pub fn current_level(&self) -> &Level {
        self.pack
            .get_level(&self.current_level)
            .expect("current_level is checked")
    }

    pub fn current_level_mut(&mut self) -> &mut Level {
        self.pack
            .get_level_mut(&self.current_level)
            .expect("current_level is checked")
    }

    pub fn current_key(&self) -> &str {
        &self.current_level
    }

    pub fn update(&mut self, dir: Direction) -> Vec<Event> {
        let result = self.current_level_mut().update(dir);
        self.selected = result.selected;
        if result.status == TurnStatus::Win {
            println!("You win!");
            self.return_to_root();
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

    pub fn return_to_root(&mut self) {
        let root = self.pack.root().to_string();
        self.goto(&root)
    }

    fn goto(&mut self, key: &str) {
        debug_assert!(self.pack.get_level(key).is_some());
        self.current_level = key.to_string();
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
