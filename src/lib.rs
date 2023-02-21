use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell},
    rc::Rc,
};
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

struct Position {}

impl Component for Position {}

type ComponentBitmap = u64;
type EntityComponentMap = u64;
type EntityId = usize;

pub struct World {
    component_manager: component::ComponentManager,
    resource_manager: resource::ResourceManager,
}

impl World {
    pub fn new() -> Self {
        Self::default()
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
    pub fn create_resource<T: Resource + 'static>(&mut self, resource: T) -> ResourceIndex {
        self.resource_manager.create_resource(resource)
    }
    pub fn get_resource<T: Resource + 'static>(&self, index: ResourceIndex) -> Ref<T> {
        self.resource_manager.get_resource::<T>(index)
    }
}
impl Default for World {
    fn default() -> Self {
        Self {
            component_manager: Default::default(),
            resource_manager: Default::default(),
        }
    }
}
