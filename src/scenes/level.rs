use crate::{
    eval::{Event, TurnStatus},
    level::{Level, LevelKind},
    lex::lex,
    pack::Levelpack,
    parse::parse,
    query::query_selected,
    world::Direction,
};

#[derive(Debug)]
pub enum Nav {
    /// Entering this level key
    Enter(String),
    /// Backing out of the current level
    Back,
}

#[derive(Clone, Copy)]
pub enum LevelAction {
    Move(Direction),
    Idle,
    Undo,
    EnterLevel,
    BackOutOfLevel,
    Restart,
}

#[derive(Debug)]
pub struct LevelScene {
    pack: Levelpack,
    selected: Vec<u64>,
    visited: Vec<String>,
    nav: Option<Nav>,
}

impl LevelScene {
    pub fn new(pack: Levelpack) -> Self {
        let root = pack.root().to_string();
        let mut game = Self {
            pack,
            selected: Vec::new(),
            visited: vec![root],
            nav: None,
        };
        game.update_selected();
        game
    }

    pub fn from_file(pack_path: &str) -> Result<Self, String> {
        let pack = LevelScene::read_pack(pack_path)?;
        Ok(Self::new(pack))
    }

    pub fn reload(&mut self, path: &str) -> Result<(), String> {
        let root = self.visited.first().expect("visited cannot be empty");
        self.pack = LevelScene::read_pack(path)?;

        // reset navigation
        self.nav = None;

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

    pub fn update(&mut self, action: LevelAction) -> Vec<Event> {
        if self.nav.is_some() {
            return vec![];
        }
        match action {
            LevelAction::Idle => return self.tick(None),
            LevelAction::Move(dir) => return self.tick(Some(dir)),
            LevelAction::BackOutOfLevel => self.return_to_parent(),
            LevelAction::EnterLevel => self.enter_link(),
            LevelAction::Undo => self.undo(),
            LevelAction::Restart => self.restart(),
        }
        vec![]
    }

    fn restart(&mut self) {
        self.current_level_mut().restart();
        self.update_selected();
    }

    fn tick(&mut self, direction: Option<Direction>) -> Vec<Event> {
        let result = self.current_level_mut().update(direction);
        self.selected = result.selected;
        if result.status == TurnStatus::Win {
            println!("You win!");
            self.return_to_parent();
        }
        result.events
    }

    pub fn caption(&self) -> Option<&str> {
        if let Some(link) = self.current_level().link_for(&self.selected)
            && let Some(level) = self.pack.get_level(link.key())
        {
            return Some(level.name());
        }
        let level = self.current_level();
        if level.kind() == LevelKind::Puzzle {
            return Some(level.name());
        }
        None
    }

    pub fn enter_link(&mut self) {
        let Some(link) = self.current_level().link_for(&self.selected) else {
            return;
        };
        let link = link.key().to_string();
        self.nav = Some(Nav::Enter(link));
    }

    pub fn return_to_parent(&mut self) {
        if self.visited.len() > 1 {
            self.nav = Some(Nav::Back);
        }
    }

    pub fn pending(&self) -> &Option<Nav> {
        &self.nav
    }

    pub fn card(&self) -> Option<&str> {
        match &self.nav {
            Some(Nav::Enter(key)) => {
                if let Some(level) = self.pack.get_level(key) {
                    Some(level.name())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn commit(&mut self) {
        match self.nav.take() {
            Some(Nav::Enter(key)) => self.goto(&key),
            Some(Nav::Back) => self.leave(),
            None => (),
        }
    }

    fn goto(&mut self, key: &str) {
        debug_assert!(self.pack.get_level(key).is_some());
        self.visited.push(key.to_string());
        self.update_selected();
    }

    fn leave(&mut self) {
        if self.current_level().kind() == LevelKind::Puzzle {
            self.current_level_mut().restart();
        }
        self.visited.pop();
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

#[cfg(test)]
mod tests {
    use super::*;

    const PACK: &str = "
        Pack = Test Pack
        Root = root

        [root]
        Name = Map
        Kind = map
        Link = 0,0,puzzle
        Data =
        cu/le

        [puzzle]
        Name = puzzle
        Data =
        BA IS YO ba ..
    ";

    #[test]
    fn test_return_to_parent_resets_level() {
        let mut scene = LevelScene::new(Levelpack::parse(PACK).unwrap());
        scene.update(LevelAction::EnterLevel);
        let initial = scene.current_level().grid().to_ascii();
        scene.update(LevelAction::Move(Direction::East));
        scene.update(LevelAction::BackOutOfLevel);
        scene.update(LevelAction::EnterLevel);
        assert_eq!(scene.current_level().grid().to_ascii(), initial);
    }
}
