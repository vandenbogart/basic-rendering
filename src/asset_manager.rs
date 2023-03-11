use uuid::Uuid;

use std::{any::Any, collections::HashMap};

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct AssetHandle(Uuid);

pub struct Asset<T> {
    pub asset_handle: AssetHandle,
    pub asset: T,
}

pub struct AssetManager {
    assets: HashMap<AssetHandle, Box<dyn Any>>,
}

impl AssetManager {
    pub fn new() -> AssetManager {
        AssetManager::default()
    }

    pub fn create_asset<T: Any + 'static>(&mut self, asset: T) -> AssetHandle {
        let asset_handle = AssetHandle(Uuid::new_v4());
        let asset = Asset {
            asset_handle,
            asset,
        };
        self.assets.insert(asset_handle, Box::new(asset));
        asset_handle
    }

    pub fn get_asset<T: Any + 'static>(&self, handle: AssetHandle) -> Option<&Asset<T>> {
        if let Some(asset) = self.assets.get(&handle) {
            asset.as_ref().downcast_ref::<Asset<T>>()
        } else {
            None
        }
    }
}
impl Default for AssetManager {
    fn default() -> Self {
        Self {
            assets: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAsset {
        value: u32,
    }

    #[test]
    fn create_asset() {
        let asset = TestAsset { value: 100 };
        let mut asset_manager = AssetManager::new();
        let asset_handle = asset_manager.create_asset(asset);
        let fetched = asset_manager.get_asset::<TestAsset>(asset_handle);
        assert_eq!(fetched.unwrap().asset.value, 100);
    }
}
