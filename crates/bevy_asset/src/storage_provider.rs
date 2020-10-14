use std::{ io, fmt, path::Path, sync::Arc };

use async_trait::async_trait;

use super::AssetLoadError;

type Result<T> = std::result::Result<T, AssetLoadError>;

pub enum AssetStorage<'a> {
    Boxed(Vec<u8>),
    Borrowed(&'a [u8]),
    Static(&'static [u8]),
}

impl<'a> AssetStorage<'a>
{
    pub fn into_vec (self) -> Vec<u8> {
        match self {
            Self::Boxed   (value) => value,
            Self::Borrowed(value) => value.into (),
            Self::Static  (value) => value.into (),
        }
    }

    pub fn as_slice (&self) -> &[u8] {
        match self {
            Self::Boxed   (value) => &value[..],
            Self::Borrowed(value) => value,
            Self::Static  (value) => value,
        }
    }
}

#[async_trait]
pub trait AssetStorageProvider : Send + Sync + 'static {

    fn find_asset_storage_sync<'a> (&'a self, path: &Path) -> Result<Option<AssetStorage<'a>>>;

    async fn find_asset_storage<'a> (&'a self, path: &Path) -> Result<Option<AssetStorage<'a>>> {
        self.find_asset_storage_sync(path)
    }
}

type StorageProviders = Vec<Arc<dyn AssetStorageProvider>>;

#[derive(Default,Clone)]
pub struct AssetStorageResolver(Arc<StorageProviders>);

impl AssetStorageResolver {

    pub fn with_provider<T>(asset_provider: T) -> Self
        where T: AssetStorageProvider
    {
        Self(Arc::new(vec![ Arc::new (asset_provider) ]))
    }

    pub fn add_provider<T>(&mut self, asset_provider: T)
        where T: AssetStorageProvider
    {
        let asset_provider = Arc::new(asset_provider);
        if let Some(providers) = Arc::get_mut(&mut self.0)  {
            providers.push (asset_provider)
        } else {
            let providers : &StorageProviders = &self.0;
            let mut new_providers = providers.clone ();
            new_providers.push (asset_provider);
            self.0 = Arc::new(new_providers);
        }
    }
}

impl AssetStorageResolver {
    pub async fn resolve <'a> (&'a self, path: &Path) -> Result <AssetStorage<'a>> {
        log::trace!("AssetStorageResolver::resolve(path={:?})", path);
        let providers : &StorageProviders = &self.0;
        if providers.len () == 0 {
            log::warn!("no storage providers available (was looking for {:?})", path);
        }
        for provider in providers {
            log::trace!("...");
            if let Some(storage) = provider.find_asset_storage (path).await? {
                return Ok(storage);
            }
        }
        Err(AssetLoadError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{}", path.display()),
        )))
    }

    pub fn resolve_sync <'a> (&'a self, path: &Path) -> Result <AssetStorage<'a>> {

        log::trace!("AssetStorageResolver::resolve_sync(path={:?})", path);
        let providers : &StorageProviders = &self.0;
        if providers.len () == 0 {
            log::warn!("no storage providers available (was looking for {:?})", path);
        }
        for provider in providers {
            log::trace!("...");
            if let Some(storage) = provider.find_asset_storage_sync (path)? {
                return Ok(storage);
            }
        }
        Err(AssetLoadError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{}", path.display()),
        )))

    }
}

impl fmt::Debug for AssetStorageResolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AssetStorageResolver")
    }
}
