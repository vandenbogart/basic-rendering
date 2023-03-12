use super::Component;

pub struct Velocity {
    pub direction: cgmath::Vector3<f32>,
    pub speed: f32,
    pub decay: f32,
}

impl Component for Velocity {

}
