use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell},
    rc::Rc,
};

use component::ComponentManager;
use renderer::{render::Renderer, Globals};
use resource::ResourceManager;
mod component;
mod query;
pub mod renderer;
mod resource;
pub mod systems;
pub mod window;

type ComponentId = TypeId;
type ComponentRef = Option<Rc<RefCell<dyn Any>>>;
type ComponentList = Vec<ComponentRef>;

pub trait Component {}

type ResourceId = TypeId;
pub type ResourceIndex = usize;
type ResourceRef = Option<Rc<RefCell<dyn Any>>>;
type ResourceList = Vec<ResourceRef>;

pub trait Resource {}

type ComponentBitmap = u64;
type EntityComponentMap = u64;
type EntityId = usize;

pub struct World {
    component_manager: component::ComponentManager,
    resource_manager: resource::ResourceManager,
    renderer: RefCell<Renderer>,
}

impl World {
    pub fn new(renderer: Renderer) -> Self {
        Self {
            component_manager: ComponentManager::default(),
            resource_manager: ResourceManager::default(),
            renderer: RefCell::new(renderer),
        }
    }
    pub fn register_component<T: Component + 'static>(&mut self) {
        self.component_manager.register_component::<T>()
    }
    pub fn spawn(&mut self) -> EntityId {
        self.component_manager.add_entity()
    }
    pub fn add_component<T: Component + 'static>(&mut self, entity_id: EntityId, component: T) {
        self.component_manager.add_component(entity_id, component);
    }
    pub fn query(&self) -> query::EntityQuery {
        self.component_manager.query()
    }
    pub async fn create_resource<T: Resource + 'static>(
        &mut self,
        resource_path: &str,
    ) -> ResourceIndex {
        let resource = self
            .renderer
            .borrow()
            .create_model_resource(resource_path.to_string())
            .await;
        self.resource_manager.create_resource(resource)
    }
    pub fn get_resource<T: Resource + 'static>(&self, index: ResourceIndex) -> Ref<T> {
        self.resource_manager.get_resource::<T>(index)
    }
    pub async fn draw(&self, view_proj: cgmath::Matrix4<f32>) {
        let globals = Globals {
            view_proj: view_proj.into(),
            ambient_strength: 0.05,
            ambient_color: [1.0, 1.0, 1.0],
        };
        self.renderer.borrow_mut().draw(self, globals).await;
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.borrow_mut().resize(width, height);
    }
}
