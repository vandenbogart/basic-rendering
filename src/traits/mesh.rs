use crate::renderer::{Model};

pub trait Rendered {
    fn get_model(self) -> Model;
}