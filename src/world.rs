use std::slice::Iter;

use uuid::Uuid;

use crate::EntityHandle;


pub struct World {
    entities: Vec<EntityHandle>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }
    pub fn spawn(&mut self) -> EntityHandle {
        let entity = EntityHandle(Uuid::new_v4());
        self.entities.push(entity);
        entity
    }
    pub fn get_entities(&self) -> Iter<EntityHandle> {
        self.entities.iter()
    }
}
