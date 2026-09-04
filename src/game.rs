use crate::{eval::TurnStatus, level::Level, pack::Levelpack, world::Direction};

pub struct Game {
    pack: Levelpack,
    current_level: String,
    selected: Vec<u64>,
}

impl Game {
    pub fn new(pack: Levelpack) -> Self {
        Self {
            current_level: pack.root().to_string(),
            pack,
            selected: Vec::new(),
        }
    }

    pub fn current_level(&self) -> &Level {
        self.pack.get_level(&self.current_level)
    }

    pub fn current_level_mut(&mut self) -> &mut Level {
        self.pack.get_level_mut(&self.current_level)
    }

    pub fn update(&mut self, dir: Direction) -> TurnStatus {
        let result = self.pack.get_level_mut(&self.current_level).update(dir);
        self.selected = result.selected;
        if result.status == TurnStatus::Win {
            println!("You win!");
            self.return_to_root();
        }
        result.status
    }

    pub fn caption(&self) -> Option<&str> {
        let key = self.current_level().link_for(&self.selected)?;
        Some(self.pack.get_level(key).name())
    }

    pub fn enter_link(&mut self) -> bool {
        if let Some(link) = self.current_level().link_for(&self.selected) {
            self.current_level = link.to_string();
            true
        } else {
            false
        }
    }

    pub fn return_to_root(&mut self) {
        self.current_level = self.pack.root().to_string();
    }
}
