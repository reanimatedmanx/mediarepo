use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use parking_lot::Mutex;
use tauri::async_runtime::RwLock;

use crate::client_api::ApiClient;
use crate::tauri_plugin::error::{PluginError, PluginResult};
use crate::tauri_plugin::settings::{load_settings, Repository, Settings};

pub struct ApiState {
    inner: Arc<RwLock<Option<ApiClient>>>,
}

impl ApiState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_api(&self, client: ApiClient) {
        let mut inner = self.inner.write().await;
        let old_client = mem::replace(&mut *inner, Some(client));

        if let Some(client) = old_client {
            let _ = client.exit().await;
        }
    }

    pub async fn api(&self) -> PluginResult<ApiClient> {
        let inner = self.inner.read().await;
        inner
            .clone()
            .ok_or_else(|| PluginError::from("Not connected"))
    }
}

pub struct OnceBuffer {
    pub mime: String,
    pub buf: Vec<u8>,
}

impl OnceBuffer {
    pub fn new(mime: String, buf: Vec<u8>) -> Self {
        Self { mime, buf }
    }
}

#[derive(Default)]
pub struct BufferState {
    pub buffer: Arc<Mutex<HashMap<String, OnceBuffer>>>,
}

pub struct AppState {
    pub active_repo: Arc<RwLock<Option<Repository>>>,
    pub settings: Arc<RwLock<Settings>>,
}

impl AppState {
    #[tracing::instrument(level = "debug")]
    pub fn load() -> PluginResult<Self> {
        let settings = load_settings()?;

        let state = Self {
            active_repo: Arc::new(RwLock::new(None)),
            settings: Arc::new(RwLock::new(settings)),
        };

        Ok(state)
    }
}
