use crate::config::Config;
use pebble_crypto::CryptoService;
use pebble_search::TantivySearch;
use pebble_store::Store;
use std::path::PathBuf;
use std::sync::Arc;

pub type AppStateRef = Arc<AppState>;

pub struct AppState {
    pub config: Config,
    pub store: Arc<Store>,
    pub search: Arc<TantivySearch>,
    pub crypto: Arc<CryptoService>,
    pub attachments_dir: PathBuf,
}

impl AppState {
    pub fn init(config: Config) -> Result<Self, String> {
        std::fs::create_dir_all(&config.data_dir)
            .map_err(|e| format!("Failed to create data dir: {e}"))?;
        std::fs::create_dir_all(config.attachments_dir())
            .map_err(|e| format!("Failed to create attachments dir: {e}"))?;

        let store = Store::open(&config.db_path())
            .map_err(|e| format!("Failed to open store: {e}"))?;

        let search = TantivySearch::open(&config.index_dir())
            .map_err(|e| format!("Failed to open search index: {e}"))?;

        let key_file = config.key_file_path();
        let crypto = CryptoService::init(Some(&key_file))
            .map_err(|e| format!("Failed to init crypto: {e}"))?;

        let attachments_dir = config.attachments_dir();

        Ok(Self {
            config,
            store: Arc::new(store),
            search: Arc::new(search),
            crypto: Arc::new(crypto),
            attachments_dir,
        })
    }
}
