use super::Component;

pub struct WalkTo {
    pub target: Option<cgmath::Point3<f32>>,
    pub speed: f32,
}
impl WalkTo {
    pub fn new(target: Option<cgmath::Point3<f32>>, speed: f32) -> Self {
        WalkTo {
            target,
            speed,
        }
    }
}
impl Component for WalkTo {

}

