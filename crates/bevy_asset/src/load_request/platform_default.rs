use super::{ChannelAssetHandler, LoadRequest};
use crate::{AssetServerError,AssetLoadError, AssetLoader, AssetResult, Handle};
use anyhow::Result;
use async_trait::async_trait;
use crossbeam_channel::Sender;
use std::{path::{Path,PathBuf},fs::File, io, io::Read, io::ErrorKind::NotFound};

/// construct the default asset storage provider

use crate::{AssetStorageProvider, AssetStorage};

pub struct DefaultStorageProvider(PathBuf);

pub fn default_repository_root_path() -> Result<PathBuf, AssetServerError> {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        Ok(PathBuf::from(manifest_dir))
    } else {
        match std::env::current_exe() {
            Ok(exe_path) => exe_path
                .parent()
                .ok_or(AssetServerError::InvalidRootPath)
                .map(|exe_parent_path| exe_parent_path.to_owned()),
            Err(err) => Err(AssetServerError::Io(err)),
        }
    }
}

impl Default for DefaultStorageProvider {
    fn default () -> Self {
        Self(default_repository_root_path().unwrap ())
    }
}

#[async_trait]
impl AssetStorageProvider for DefaultStorageProvider
{
    //async fn find_asset_storage<'a> (&'a self, path: &Path) -> std::result::Result<Option<AssetStorage<'a>>, AssetLoadError> {
    //    self.find_asset_storage_sync(path)
    //}

    fn find_asset_storage_sync<'a> (&'a self, path: &Path) -> std::result::Result<Option<AssetStorage<'a>>, AssetLoadError>
    {
        let full_path = self.0.join (path);

        log::trace!("DefaultStorageProvider::find_asset_storage({:?})", path);

        match File::open(path) {
            Ok(mut file) => {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                log::trace!("DefaultStorageProvider::find_asset_storage - found! -> {} bytes", bytes.len ());
                Ok(Some(AssetStorage::Boxed(bytes)))
            }
            Err(e) => if e.kind () == NotFound {
                log::trace!("DefaultStorageProvider::find_asset_storage - next! -> {:?}", e);
                Ok(None)
            } else {
                log::trace!("DefaultStorageProvider::find_asset_storage - error! -> {:?}", e);
                Err(AssetLoadError::Io(io::Error::new(
                    e.kind(),
                    format!("{}", path.display()),
                )))
            }
        }
    }
}

/// Handles load requests from an AssetServer

#[async_trait]
pub trait AssetLoadRequestHandler: Send + Sync + 'static {
    async fn handle_request(&self, load_request: &LoadRequest);
    fn extensions(&self) -> &[&str];
}

impl<TLoader, TAsset> ChannelAssetHandler<TLoader, TAsset>
where
    TLoader: AssetLoader<TAsset>,
{
    pub fn new(loader: TLoader, sender: Sender<AssetResult<TAsset>>) -> Self {
        ChannelAssetHandler { sender, loader }
    }

    async fn load_asset(&self, load_request: &LoadRequest) -> Result<TAsset, AssetLoadError> {

        let storage = load_request.resolver.resolve(&load_request.path).await?;

        let asset = self.loader.from_storage(&load_request.path, storage)?;

        Ok(asset)
    }
}

#[async_trait]
impl<TLoader, TAsset> AssetLoadRequestHandler for ChannelAssetHandler<TLoader, TAsset>
where
    TLoader: AssetLoader<TAsset> + 'static,
    TAsset: Send + 'static,
{
    async fn handle_request(&self, load_request: &LoadRequest) {
        log::trace!("ChannelAssetHandler::handle_request(load_request: {{path={:?}}})", load_request.path);
        let result = self.load_asset(load_request).await;
        let asset_result = AssetResult {
            handle: Handle::from(load_request.handle_id),
            result,
            path: load_request.path.clone(),
            version: load_request.version,
        };
        log::trace!("ChannelAssetHandler::handle_request -> err: {:?}", asset_result.result.as_ref ().err());
        self.sender
            .send(asset_result)
            .expect("loaded asset should have been sent");
    }

    fn extensions(&self) -> &[&str] {
        self.loader.extensions()
    }
}
