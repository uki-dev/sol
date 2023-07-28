pub trait System {
  fn resize(&mut self, _size: (u32, u32)) {}
}

pub struct Game<'a> {
    pub systems: Vec<Box<dyn System + 'a>>,
}

impl<'a> Game<'a> {
    pub fn new() -> Game<'a> {
        Game { systems: vec![] }
    }

    pub fn resize(&mut self, size: (u32, u32)) {
      for system in self.systems.iter_mut() {
        system.resize(size);
      }
    }
}
