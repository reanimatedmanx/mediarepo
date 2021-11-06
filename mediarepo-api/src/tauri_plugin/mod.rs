use tauri::plugin::Plugin;
use tauri::{AppHandle, Builder, Invoke, Manager, Runtime};

use state::ApiState;

use crate::tauri_plugin::state::{AppState, BufferState};
use std::thread;
use std::time::Duration;

pub(crate) mod commands;
pub mod custom_schemes;
pub mod error;
mod settings;
mod state;

use commands::*;

pub fn register_plugin<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    let repo_plugin = MediarepoPlugin::new();

    custom_schemes::register_custom_uri_schemes(builder.plugin(repo_plugin))
}

pub struct MediarepoPlugin<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> MediarepoPlugin<R> {
    pub fn new() -> Self {
        Self {
            invoke_handler: Box::new(tauri::generate_handler![
                get_all_files,
                find_files,
                read_file_by_hash,
                get_file_thumbnails,
                read_thumbnail,
                get_repositories,
                get_all_tags,
                get_tags_for_file,
                get_tags_for_files,
                get_active_repository,
                add_repository,
                select_repository,
                init_repository,
                start_daemon,
                check_daemon_running,
                stop_daemon,
                disconnect_repository,
                close_local_repository,
                check_local_repository_exists,
                remove_repository,
                change_file_tags,
                create_tags,
                update_file_name
            ]),
        }
    }
}

impl<R: Runtime> Plugin<R> for MediarepoPlugin<R> {
    fn name(&self) -> &'static str {
        "mediarepo"
    }

    #[tracing::instrument(skip(self, app, _config))]
    fn initialize(
        &mut self,
        app: &AppHandle<R>,
        _config: serde_json::value::Value,
    ) -> tauri::plugin::Result<()> {
        let api_state = ApiState::new();
        app.manage(api_state);

        let buffer_state = BufferState::default();
        app.manage(buffer_state.clone());

        let repo_state = AppState::load()?;
        app.manage(repo_state);

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(10));
            buffer_state.clear_expired();
        });

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    fn extend_api(&mut self, message: Invoke<R>) {
        (self.invoke_handler)(message)
    }
}
