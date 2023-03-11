use std::{
    any::{Any, TypeId},
    cell::{RefCell, RefMut, Ref},
    collections::HashMap,
};

use crate::{EntityHandle, components::Component};

pub struct ComponentManager {
    components: HashMap<TypeId, HashMap<EntityHandle, Box<RefCell<dyn Any>>>>,
}
impl ComponentManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_component<T: Component + 'static>(&mut self) {
        let id = TypeId::of::<T>();
        self.components
            .insert(id, HashMap::new());
    }
    pub fn add_component<T: Component + 'static>(&mut self, component: T, entity: EntityHandle) {
        let id = TypeId::of::<T>();
        if let Some(ent_comp_map) = self.components.get_mut(&id) {
            ent_comp_map.insert(entity, Box::new(RefCell::new(component)));
        } else {
            panic!("Attempted to add an un-registered component");
        }
    }
    pub fn get_entity_component<T: Component + 'static>(&self, entity: EntityHandle) -> Option<Ref<T>> {
        let id = TypeId::of::<T>();
        if let Some(ent_comp_map) = self.components.get(&id) {
            match ent_comp_map.get(&entity) {
                Some(comp) => Some(Ref::map(comp.borrow(), |c| c.downcast_ref::<T>().unwrap())),
                None => None,
            }
        } else {
            panic!("Attempted to get an un-registered component");
        }
    }
    pub fn mut_entity_component<T: Component + 'static>(&self, entity: EntityHandle) -> Option<RefMut<T>> {
        let id = TypeId::of::<T>();
        if let Some(ent_comp_map) = self.components.get(&id) {
            match ent_comp_map.get(&entity) {
                Some(comp) => Some(RefMut::map(comp.borrow_mut(), |c| c.downcast_mut::<T>().unwrap())),
                None => None,
            }
        } else {
            panic!("Attempted to mutate an un-registered component");
        }
    }
    pub fn get_all_by_type<T: Component + 'static>(&self) -> Vec<(EntityHandle, Ref<T>)> {
        let id = TypeId::of::<T>();
        if let Some(ent_comp_map) = self.components.get(&id) {
            ent_comp_map
                .iter()
                .map(|(k, v)| (*k, Ref::map(v.borrow(), |c| c.downcast_ref::<T>().unwrap())))
                .collect()
        } else {
            panic!("Attempted to get component list for unregistered component");
        }
    }
}
impl Default for ComponentManager {
    fn default() -> Self {
        Self {
            components: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
}
