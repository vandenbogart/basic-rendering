use std::{
    any::TypeId,
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{Resource, ResourceId, ResourceIndex, ResourceList};

pub struct ResourceManager {
    resource_store: HashMap<ResourceId, ResourceList>,
}
impl ResourceManager {
    pub fn new() -> ResourceManager {
        ResourceManager::default()
    }

    pub fn create_resource<T: Resource + 'static>(&mut self, resource: T) -> ResourceIndex {
        let id = TypeId::of::<T>();
        match self.resource_store.get_mut(&id) {
            Some(list) => {
                list.push(Some(Rc::new(RefCell::new(resource))));
                list.len() - 1
            }
            None => {
                self.resource_store.insert(id, Default::default());
                self.create_resource(resource)
            }
        }
    }

    pub fn get_resource<T: Resource + 'static>(&self, index: ResourceIndex) -> Ref<T> {
        let id = TypeId::of::<T>();
        let resource_list = self
            .resource_store
            .get(&id)
            .expect("Resource Type does not exist");
        let resource = resource_list
            .get(index)
            .expect("Resource cannot be found at index");
        let borrow = resource.as_ref().unwrap().borrow();
        Ref::map(borrow, |any| any.downcast_ref().unwrap())
    }
}
impl Default for ResourceManager {
    fn default() -> Self {
        Self {
            resource_store: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestResource {
        value: u32,
    }
    impl Resource for TestResource {}

    #[test]
    fn create_get_resource() {
        let resource = TestResource { value: 100 };
        let mut rm = ResourceManager::new();
        let resource_index = rm.create_resource(resource);
        let fetched = rm.get_resource::<TestResource>(resource_index);
        assert_eq!(fetched.value, 100);
    }
}
