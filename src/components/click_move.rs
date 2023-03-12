use super::Component;

pub struct ClickMove {
    pub target: Option<cgmath::Point3<f32>>,
    pub speed: f32,
}
impl ClickMove {
    pub fn new(speed: f32) -> Self {
        Self {
            target: None,
            speed,
        }
    }
}


impl Component for ClickMove {}
