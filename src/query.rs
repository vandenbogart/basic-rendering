use std::cell::{Ref, RefMut};

use crate::{component::ComponentManager, Component, EntityComponentMap, EntityId};

pub struct EntityQueryResult<'a> {
    entities: Vec<EntityId>,
    component_manager: &'a ComponentManager,
}
impl<'a> EntityQueryResult<'a> {
    pub fn get_entities(&self) -> &Vec<EntityId> {
        &self.entities
    }
    pub fn get_component<T: Component + 'static>(&self, entity: EntityId) -> Ref<T> {
        self.component_manager.get_component::<T>(entity)
    }
    pub fn get_component_mut<T: Component + 'static>(&self, entity: EntityId) -> RefMut<T> {
        self.component_manager.get_component_mut::<T>(entity)
    }
}

pub struct EntityQuery<'a> {
    query_bitmap: EntityComponentMap,
    component_manager: &'a ComponentManager,
}

impl<'a> EntityQuery<'a> {
    pub fn new(cm: &'a ComponentManager) -> Self {
        EntityQuery {
            query_bitmap: 0,
            component_manager: cm,
        }
    }
    pub fn with_component<T: Component + 'static>(mut self) -> Self {
        let c_bitmap = self.component_manager.get_component_bitmap::<T>();
        self.query_bitmap |= c_bitmap;
        self
    }
    pub fn execute(self) -> EntityQueryResult<'a> {
        let entities = self
            .component_manager
            .get_entities_with_components(self.query_bitmap);
        EntityQueryResult {
            entities,
            component_manager: self.component_manager,
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
    #[test]
    fn create_query() {
        let mut cm = ComponentManager::new();
        cm.register_component::<DummyComponent>();
        cm.register_component::<DummyComponent2>();
        let query = cm.query().with_component::<DummyComponent2>();
        assert_eq!(query.query_bitmap, 2);
    }

    #[test]
    fn get_entities_with_components() {
        let mut cm = ComponentManager::new();
        cm.register_component::<DummyComponent>();
        cm.register_component::<DummyComponent2>();
        let entity1 = cm.add_entity();
        let entity2 = cm.add_entity();
        cm.add_component(entity1, DummyComponent {});
        cm.add_component(entity1, DummyComponent2 {});
        cm.add_component(entity2, DummyComponent2 {});
        let result = cm
            .query()
            .with_component::<DummyComponent>()
            .with_component::<DummyComponent2>()
            .execute();
        assert_eq!(result.entities.contains(&entity1), true);
        let result = cm.query().with_component::<DummyComponent2>().execute();
        assert_eq!(result.entities.contains(&entity1), true);
        assert_eq!(result.entities.contains(&entity2), true);
    }
}
