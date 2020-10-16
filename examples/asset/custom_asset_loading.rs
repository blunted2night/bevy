
use bevy::{
    prelude::*,
    asset::{
        AssetLoader,
        AssetStorage,
        AssetLoadError,
        AssetStorageProvider
    }
};

use ron::de::from_bytes;
use serde::Deserialize;
use std::path::Path;
use async_trait::async_trait;

#[derive(Deserialize)]
pub struct MyCustomData {
    pub num: i32,
}

#[derive(Deserialize)]
pub struct MySecondCustomData {
    pub is_set: bool,
}

// create a custom loader for data files
#[derive(Default)]
pub struct DataFileLoader {
    matching_extensions: Vec<&'static str>,
}

impl DataFileLoader {
    pub fn from_extensions(matching_extensions: Vec<&'static str>) -> Self {
        DataFileLoader {
            matching_extensions,
        }
    }
}

impl<TAsset> AssetLoader<TAsset> for DataFileLoader
where
    for<'de> TAsset: Deserialize<'de>,
{
    fn from_storage(&self, _asset_path: &Path, storage: AssetStorage) -> Result<TAsset, anyhow::Error> {
        Ok(from_bytes::<TAsset>(storage.as_slice())?)
    }

    fn extensions(&self) -> &[&str] {
        self.matching_extensions.as_slice()
    }
}

// create a custom storage provider for accessing files
pub struct StaticStorageProvider(&'static [(&'static str, &'static [u8])]);

#[async_trait]
impl AssetStorageProvider for StaticStorageProvider {
    fn find_asset_storage_sync<'a> (&'a self, path: &Path)
        -> std::result::Result<Option<AssetStorage<'a>>, AssetLoadError>
    {
        for (entry_path, value) in self.0 {
            if Path::new(entry_path) == path {
                return Ok(Some(AssetStorage::Static(value)));
            }
        }

        Ok(None)
    }
}

const RESOURCES : &'static [(&'static str, &'static [u8])] = &[
    ("embedded.data1", std::include_bytes!{"embedded.data1"}),
];

/// This example illustrates various ways to load assets
fn main() {

    App::build()
        .add_default_plugins()
        .add_asset::<MyCustomData>()
        .add_asset_storage_provider (StaticStorageProvider(RESOURCES))
        .add_asset_loader_from_instance::<MyCustomData, DataFileLoader>(
            DataFileLoader::from_extensions(vec!["data1"]),
        )
        .add_asset::<MySecondCustomData>()
        .add_asset_loader_from_instance::<MySecondCustomData, DataFileLoader>(
            DataFileLoader::from_extensions(vec!["data2"]),
        )
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut data1s: ResMut<Assets<MyCustomData>>,
    mut data2s: ResMut<Assets<MySecondCustomData>>,
) {
    let data1_handle = asset_server
        .load_sync(&mut data1s, "assets/data/test_data.data1")
        .unwrap();
    let data2_handle = asset_server
        .load_sync(&mut data2s, "assets/data/test_data.data2")
        .unwrap();
    let embedded_data1_handle = asset_server
        .load_sync(&mut data1s, "embedded.data1")
        .unwrap();

    let data1 = data1s.get(&data1_handle).unwrap();
    println!("Data 1 loaded with value {}", data1.num);

    let data2 = data2s.get(&data2_handle).unwrap();
    println!("Data 2 loaded with value {}", data2.is_set);

    let embedded_data1 = data1s.get(&embedded_data1_handle).unwrap();
    println!("embedded Data 1 loaded with value {}", embedded_data1.num);
}
