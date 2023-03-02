use crate::{traits::{entity::Entity, mesh::{Rendered}}, renderer::{Model}};
impl Entity for Player {}

pub struct Player {
    model: Model
}
impl Player {
    pub fn new(model: Model) -> Self {
        Self {
            model,
        }
    }
}
impl Rendered for Player {
    fn get_model(&self) -> Model {
        self.model
    }
}
