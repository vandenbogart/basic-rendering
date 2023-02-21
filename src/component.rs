use std::{collections::HashMap, any::TypeId, rc::Rc, cell::{RefCell, RefMut, Ref}};

use crate::{EntityComponentMap, ComponentId, ComponentList, ComponentBitmap, Component, EntityId};

use crate::query::{EntityQuery};

pub struct ComponentManager {
    entities: Vec<EntityComponentMap>,
    components: HashMap<ComponentId, ComponentList>,
    component_bitmaps: HashMap<ComponentId, ComponentBitmap>,
    num_component_types: u32,
}
impl ComponentManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_component<T: Component + 'static>(&mut self) {
        let id = TypeId::of::<T>();
        let bitmap = (1 as ComponentBitmap) << self.num_component_types;
        self.components.insert(id, Vec::new());
        self.component_bitmaps.insert(id, bitmap);
        self.num_component_types += 1;
        if self.num_component_types > 64 {
            panic!("Exceeded maximum number of registered component types")
        }
    }

    pub fn get_component_bitmap<T: Component + 'static>(&self) -> ComponentBitmap{
        let id = TypeId::of::<T>();
        self.component_bitmaps.get(&id).expect("Attempted to access component which was not registered").clone()
    }

    pub fn add_entity(&mut self) -> EntityId {
        if self.num_component_types == 0 {
            panic!("Attempted to register an entity before registering component types.");
        }
        self.components.values_mut().for_each(|list| {
            list.push(Default::default());
        });
        self.entities.push(0);
        self.entities.len() - 1
    }

    pub fn add_component<T: Component + 'static>(&mut self, entity_id: EntityId, component: T) {
        let component_id = TypeId::of::<T>();
        let component_list = self
            .components
            .get_mut(&component_id)
            .expect("Attempted to add unregistered component.");
        let entity_component_slot = component_list
            .get(entity_id)
            .expect("Attempted to add component to non-existant entity");
        match entity_component_slot {
            Some(_) => panic!("Attempted to add component to slot which was already full"),
            None => component_list.insert(entity_id, Some(Rc::new(RefCell::new(component)))),
        };
        self.entities[entity_id] |= self.component_bitmaps[&component_id];
    }
    pub fn get_component<T: Component + 'static>(&self, entity: EntityId) -> Ref<T> {
        let c_id = TypeId::of::<T>();
        let components = self.components.get(&c_id).expect("Attempted to get component which was not registered");
        let component_data = components[entity].as_ref().expect("Attempted to get component for entity which did not have it").borrow();
        Ref::map(component_data,|any| {
            any.downcast_ref::<T>().unwrap()
        })
    }
    pub fn get_component_mut<T: Component + 'static>(&self, entity: EntityId) -> RefMut<T> {
        let c_id = TypeId::of::<T>();
        let components = self.components.get(&c_id).expect("Attempted to get component which was not registered");
        let component_data = components[entity].as_ref().expect("Attempted to get component for entity which did not have it").borrow_mut();
        RefMut::map(component_data,|any| {
            any.downcast_mut::<T>().unwrap()
        })
    }
    pub fn get_entities_with_components(
        &self,
        component_bitmap: EntityComponentMap,
    ) -> Vec<EntityId> {
        self.entities
            .iter()
            .enumerate()
            .filter_map(|(entity_id, entity_bitmap)| {
                if entity_bitmap & component_bitmap == component_bitmap {
                    Some(entity_id)
                } else {
                    None
                }
            }).collect()
    }
    pub fn query(&self) -> EntityQuery {
        EntityQuery::new(self)
    }
}
impl Default for ComponentManager {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            components: Default::default(),
            component_bitmaps: Default::default(),
            num_component_types: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct DummyComponent {}
    impl Component for DummyComponent {}
    struct DummyComponent2 {}
    impl Component for DummyComponent2 {}

    struct DataComponent {
        value: usize,
    }
    impl Component for DataComponent {}
    #[test]
    fn register_component() {
        let mut cm = ComponentManager::new();
        cm.register_component::<DummyComponent>();
        assert_eq!(cm.num_component_types, 1);
        assert_eq!(
            cm.components.contains_key(&TypeId::of::<DummyComponent>()),
            true
        );
        assert_eq!(
            cm.component_bitmaps
                .contains_key(&TypeId::of::<DummyComponent>()),
            true
        );
    }

    #[test]
    #[should_panic]
    fn add_entity_before_registering_component() {
        let mut cm = ComponentManager::new();
        cm.add_entity();
    }

    #[test]
    fn add_entity() {
        let mut cm = ComponentManager::new();
        cm.register_component::<DummyComponent>();
        let entity_id = cm.add_entity();
        assert_eq!(entity_id, 0);
        assert_eq!(cm.entities.len(), 1);
    }

    #[test]
    fn add_component() {
        let mut cm = ComponentManager::new();
        cm.register_component::<DummyComponent>();
        cm.register_component::<DummyComponent2>();
        let entity = cm.add_entity();
        let component = DummyComponent {};
        cm.add_component(entity, component);
        let c_type = TypeId::of::<DummyComponent>();
        let c_bitmask = cm.component_bitmaps[&c_type];

        assert_eq!(c_bitmask == 1, true);
        assert_eq!(cm.components[&c_type][entity].is_some(), true);
        assert_eq!(cm.entities[entity] & c_bitmask == 1, true);

        let component = DummyComponent2 {};
        cm.add_component(entity, component);
        let c_type = TypeId::of::<DummyComponent2>();
        let c_bitmask = cm.component_bitmaps[&c_type];

        assert_eq!(c_bitmask == 2, true);
        assert_eq!(cm.components[&c_type][entity].is_some(), true);
        assert_eq!(cm.entities[entity] == 3, true);
    }


    
    #[test]
    fn get_component() {
        let mut cm = ComponentManager::new();
        cm.register_component::<DataComponent>();
        let entity1 = cm.add_entity();
        cm.add_component(entity1, DataComponent { value: 100 });
        let component = cm.get_component::<DataComponent>(entity1);
        assert_eq!(component.value, 100);
    }

    #[test]
    fn get_component_mut() {
        let mut cm = ComponentManager::new();
        cm.register_component::<DataComponent>();
        let entity1 = cm.add_entity();
        cm.add_component(entity1, DataComponent { value: 100 });
        let mut component = cm.get_component_mut::<DataComponent>(entity1);
        component.value = 1;
        drop(component);
        let component = cm.get_component::<DataComponent>(entity1);
        assert_eq!(component.value, 1);
    }

}
