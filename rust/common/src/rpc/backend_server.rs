use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;
use tonic::transport::Server;
use tonic::Request;
use tonic::Response;
use tonic::Status;

use crate::model::DownloadStatus;
use crate::model::EntrypointId;
use crate::model::LocalSaveData;
use crate::model::PhysicalKey;
use crate::model::PhysicalShortcut;
use crate::model::PluginId;
use crate::model::PluginPreferenceUserData;
use crate::model::SettingsEntrypointType;
use crate::model::SettingsPlugin;
use crate::model::SettingsTheme;
use crate::model::WindowPositionMode;
use crate::rpc::grpc::rpc_backend_server::RpcBackend;
use crate::rpc::grpc::rpc_backend_server::RpcBackendServer;
use crate::rpc::grpc::RpcDownloadPluginRequest;
use crate::rpc::grpc::RpcDownloadPluginResponse;
use crate::rpc::grpc::RpcDownloadStatus;
use crate::rpc::grpc::RpcDownloadStatusRequest;
use crate::rpc::grpc::RpcDownloadStatusResponse;
use crate::rpc::grpc::RpcDownloadStatusValue;
use crate::rpc::grpc::RpcEntrypoint;
use crate::rpc::grpc::RpcEntrypointTypeSettings;
use crate::rpc::grpc::RpcGeneratedEntrypoint;
use crate::rpc::grpc::RpcGetGlobalShortcutRequest;
use crate::rpc::grpc::RpcGetGlobalShortcutResponse;
use crate::rpc::grpc::RpcGetThemeRequest;
use crate::rpc::grpc::RpcGetThemeResponse;
use crate::rpc::grpc::RpcGetWindowPositionModeRequest;
use crate::rpc::grpc::RpcGetWindowPositionModeResponse;
use crate::rpc::grpc::RpcPingRequest;
use crate::rpc::grpc::RpcPingResponse;
use crate::rpc::grpc::RpcPlugin;
use crate::rpc::grpc::RpcPluginsRequest;
use crate::rpc::grpc::RpcPluginsResponse;
use crate::rpc::grpc::RpcRemovePluginRequest;
use crate::rpc::grpc::RpcRemovePluginResponse;
use crate::rpc::grpc::RpcRunActionRequest;
use crate::rpc::grpc::RpcRunActionResponse;
use crate::rpc::grpc::RpcSaveLocalPluginRequest;
use crate::rpc::grpc::RpcSaveLocalPluginResponse;
use crate::rpc::grpc::RpcSetEntrypointStateRequest;
use crate::rpc::grpc::RpcSetEntrypointStateResponse;
use crate::rpc::grpc::RpcSetGlobalShortcutRequest;
use crate::rpc::grpc::RpcSetGlobalShortcutResponse;
use crate::rpc::grpc::RpcSetPluginStateRequest;
use crate::rpc::grpc::RpcSetPluginStateResponse;
use crate::rpc::grpc::RpcSetPreferenceValueRequest;
use crate::rpc::grpc::RpcSetPreferenceValueResponse;
use crate::rpc::grpc::RpcSetThemeRequest;
use crate::rpc::grpc::RpcSetThemeResponse;
use crate::rpc::grpc::RpcSetWindowPositionModeRequest;
use crate::rpc::grpc::RpcSetWindowPositionModeResponse;
use crate::rpc::grpc::RpcShortcut;
use crate::rpc::grpc::RpcShowSettingsWindowRequest;
use crate::rpc::grpc::RpcShowSettingsWindowResponse;
use crate::rpc::grpc::RpcShowWindowRequest;
use crate::rpc::grpc::RpcShowWindowResponse;
use crate::rpc::grpc_convert::plugin_preference_to_rpc;
use crate::rpc::grpc_convert::plugin_preference_user_data_from_rpc;
use crate::rpc::grpc_convert::plugin_preference_user_data_to_rpc;

pub async fn wait_for_backend_server() {
    loop {
        let addr: SocketAddr = "127.0.0.1:42320".parse().unwrap();

        if TcpStream::connect(addr).await.is_ok() {
            return;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn start_backend_server(server: Box<dyn BackendServer + Sync + Send>) {
    let addr = "127.0.0.1:42320".parse().unwrap();

    Server::builder()
        .add_service(RpcBackendServer::new(RpcBackendServerImpl::new(server)))
        .serve(addr)
        .await
        .expect("unable to start backend server");
}

struct RpcBackendServerImpl {
    server: Box<dyn BackendServer + Sync + Send>,
}

impl RpcBackendServerImpl {
    pub fn new(server: Box<dyn BackendServer + Sync + Send>) -> Self {
        Self { server }
    }
}

#[tonic::async_trait]
pub trait BackendServer {
    async fn show_window(&self) -> anyhow::Result<()>;

    async fn show_settings_window(&self) -> anyhow::Result<()>;

    async fn run_action(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_id: String,
    ) -> anyhow::Result<()>;

    async fn plugins(&self) -> anyhow::Result<Vec<SettingsPlugin>>;

    async fn set_plugin_state(&self, plugin_id: PluginId, enabled: bool) -> anyhow::Result<()>;

    async fn set_entrypoint_state(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        enabled: bool,
    ) -> anyhow::Result<()>;

    async fn set_global_shortcut(&self, shortcut: Option<PhysicalShortcut>) -> anyhow::Result<()>;

    async fn get_global_shortcut(&self) -> anyhow::Result<(Option<PhysicalShortcut>, Option<String>)>;

    async fn set_theme(&self, theme: SettingsTheme) -> anyhow::Result<()>;

    async fn get_theme(&self) -> anyhow::Result<SettingsTheme>;

    async fn set_window_position_mode(&self, mode: WindowPositionMode) -> anyhow::Result<()>;

    async fn get_window_position_mode(&self) -> anyhow::Result<WindowPositionMode>;

    async fn set_preference_value(
        &self,
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
        preference_id: String,
        preference_value: PluginPreferenceUserData,
    ) -> anyhow::Result<()>;

    async fn download_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()>;

    async fn download_status(&self) -> anyhow::Result<HashMap<PluginId, DownloadStatus>>;

    async fn remove_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()>;

    async fn save_local_plugin(&self, path: String) -> anyhow::Result<LocalSaveData>;
}

#[tonic::async_trait]
impl RpcBackend for RpcBackendServerImpl {
    async fn ping(&self, _: Request<RpcPingRequest>) -> Result<Response<RpcPingResponse>, Status> {
        Ok(Response::new(RpcPingResponse::default()))
    }

    async fn show_window(
        &self,
        _request: Request<RpcShowWindowRequest>,
    ) -> Result<Response<RpcShowWindowResponse>, Status> {
        self.server
            .show_window()
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcShowWindowResponse::default()))
    }

    async fn show_settings_window(
        &self,
        _request: Request<RpcShowSettingsWindowRequest>,
    ) -> Result<Response<RpcShowSettingsWindowResponse>, Status> {
        self.server
            .show_settings_window()
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcShowSettingsWindowResponse::default()))
    }

    async fn run_action(
        &self,
        request: Request<RpcRunActionRequest>,
    ) -> Result<Response<RpcRunActionResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let entrypoint_id = request.entrypoint_id;
        let action_id = request.action_id;

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);

        self.server
            .run_action(plugin_id, entrypoint_id, action_id)
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcRunActionResponse::default()))
    }

    async fn plugins(&self, _: Request<RpcPluginsRequest>) -> Result<Response<RpcPluginsResponse>, Status> {
        let plugins = self
            .server
            .plugins()
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?
            .into_iter()
            .map(|plugin| {
                let entrypoints = plugin
                    .entrypoints
                    .into_iter()
                    .map(|(_, entrypoint)| {
                        RpcEntrypoint {
                            enabled: entrypoint.enabled,
                            entrypoint_id: entrypoint.entrypoint_id.to_string(),
                            entrypoint_name: entrypoint.entrypoint_name,
                            entrypoint_description: entrypoint.entrypoint_description,
                            entrypoint_type: match entrypoint.entrypoint_type {
                                SettingsEntrypointType::Command => RpcEntrypointTypeSettings::SCommand,
                                SettingsEntrypointType::View => RpcEntrypointTypeSettings::SView,
                                SettingsEntrypointType::InlineView => RpcEntrypointTypeSettings::SInlineView,
                                SettingsEntrypointType::EntrypointGenerator => {
                                    RpcEntrypointTypeSettings::SEntrypointGenerator
                                }
                            }
                            .into(),
                            preferences: entrypoint
                                .preferences
                                .into_iter()
                                .map(|(key, value)| (key, plugin_preference_to_rpc(value)))
                                .collect(),
                            preferences_user_data: entrypoint
                                .preferences_user_data
                                .into_iter()
                                .map(|(key, value)| (key, plugin_preference_user_data_to_rpc(value)))
                                .collect(),
                            generated_entrypoints: entrypoint
                                .generated_entrypoints
                                .into_iter()
                                .map(|(entrypoint_id, data)| {
                                    (
                                        entrypoint_id.to_string(),
                                        RpcGeneratedEntrypoint {
                                            entrypoint_id: data.entrypoint_id.to_string(),
                                            entrypoint_name: data.entrypoint_name,
                                        },
                                    )
                                })
                                .collect(),
                        }
                    })
                    .collect();

                RpcPlugin {
                    plugin_id: plugin.plugin_id.to_string(),
                    plugin_name: plugin.plugin_name,
                    plugin_description: plugin.plugin_description,
                    enabled: plugin.enabled,
                    entrypoints,
                    preferences: plugin
                        .preferences
                        .into_iter()
                        .map(|(key, value)| (key, plugin_preference_to_rpc(value)))
                        .collect(),
                    preferences_user_data: plugin
                        .preferences_user_data
                        .into_iter()
                        .map(|(key, value)| (key, plugin_preference_user_data_to_rpc(value)))
                        .collect(),
                }
            })
            .collect();

        Ok(Response::new(RpcPluginsResponse { plugins }))
    }

    async fn set_plugin_state(
        &self,
        request: Request<RpcSetPluginStateRequest>,
    ) -> Result<Response<RpcSetPluginStateResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let enabled = request.enabled;

        let plugin_id = PluginId::from_string(plugin_id);

        self.server
            .set_plugin_state(plugin_id, enabled)
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcSetPluginStateResponse::default()))
    }

    async fn set_entrypoint_state(
        &self,
        request: Request<RpcSetEntrypointStateRequest>,
    ) -> Result<Response<RpcSetEntrypointStateResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let entrypoint_id = request.entrypoint_id;
        let enabled = request.enabled;

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);

        self.server
            .set_entrypoint_state(plugin_id, entrypoint_id, enabled)
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcSetEntrypointStateResponse::default()))
    }

    async fn set_preference_value(
        &self,
        request: Request<RpcSetPreferenceValueRequest>,
    ) -> Result<Response<RpcSetPreferenceValueResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let plugin_id = PluginId::from_string(plugin_id);

        let entrypoint_id = if request.entrypoint_id.is_empty() {
            None
        } else {
            Some(EntrypointId::from_string(request.entrypoint_id))
        };

        let preference_id = request.preference_id;
        let preference_value = request.preference_value.unwrap();

        self.server
            .set_preference_value(
                plugin_id,
                entrypoint_id,
                preference_id,
                plugin_preference_user_data_from_rpc(preference_value),
            )
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcSetPreferenceValueResponse::default()))
    }

    async fn set_global_shortcut(
        &self,
        request: Request<RpcSetGlobalShortcutRequest>,
    ) -> Result<Response<RpcSetGlobalShortcutResponse>, Status> {
        let request = request.into_inner();

        let shortcut = request.shortcut.map(|shortcut| {
            let physical_key = shortcut.physical_key;
            let modifier_shift = shortcut.modifier_shift;
            let modifier_control = shortcut.modifier_control;
            let modifier_alt = shortcut.modifier_alt;
            let modifier_meta = shortcut.modifier_meta;

            PhysicalShortcut {
                physical_key: PhysicalKey::from_value(physical_key),
                modifier_shift,
                modifier_control,
                modifier_alt,
                modifier_meta,
            }
        });

        let error = self
            .server
            .set_global_shortcut(shortcut)
            .await
            .map_err(|err| format!("{:#}", err))
            .err();

        Ok(Response::new(RpcSetGlobalShortcutResponse { error }))
    }

    async fn get_global_shortcut(
        &self,
        _request: Request<RpcGetGlobalShortcutRequest>,
    ) -> Result<Response<RpcGetGlobalShortcutResponse>, Status> {
        let (shortcut, error) = self
            .server
            .get_global_shortcut()
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcGetGlobalShortcutResponse {
            shortcut: shortcut.map(|shortcut| {
                RpcShortcut {
                    physical_key: shortcut.physical_key.to_value(),
                    modifier_shift: shortcut.modifier_shift,
                    modifier_control: shortcut.modifier_control,
                    modifier_alt: shortcut.modifier_alt,
                    modifier_meta: shortcut.modifier_meta,
                }
            }),
            error,
        }))
    }

    async fn set_theme(&self, request: Request<RpcSetThemeRequest>) -> Result<Response<RpcSetThemeResponse>, Status> {
        let theme = request.into_inner().theme;

        let theme = match theme.as_str() {
            "AutoDetect" => SettingsTheme::AutoDetect,
            "ThemeFile" => SettingsTheme::ThemeFile,
            "Config" => SettingsTheme::Config,
            "MacOSLight" => SettingsTheme::MacOSLight,
            "MacOSDark" => SettingsTheme::MacOSDark,
            "Legacy" => SettingsTheme::Legacy,
            _ => unreachable!(),
        };

        self.server
            .set_theme(theme)
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcSetThemeResponse::default()))
    }

    async fn get_theme(&self, _request: Request<RpcGetThemeRequest>) -> Result<Response<RpcGetThemeResponse>, Status> {
        let theme = self
            .server
            .get_theme()
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        let theme = match theme {
            SettingsTheme::AutoDetect => "AutoDetect",
            SettingsTheme::ThemeFile => "ThemeFile",
            SettingsTheme::Config => "Config",
            SettingsTheme::MacOSLight => "MacOSLight",
            SettingsTheme::MacOSDark => "MacOSDark",
            SettingsTheme::Legacy => "Legacy",
        };

        Ok(Response::new(RpcGetThemeResponse {
            theme: theme.to_string(),
        }))
    }
    async fn set_window_position_mode(
        &self,
        request: Request<RpcSetWindowPositionModeRequest>,
    ) -> Result<Response<RpcSetWindowPositionModeResponse>, Status> {
        let mode = request.into_inner().mode;

        let mode = match mode.as_str() {
            "Static" => WindowPositionMode::Static,
            "ActiveMonitor" => WindowPositionMode::ActiveMonitor,
            _ => unreachable!(),
        };

        self.server
            .set_window_position_mode(mode)
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcSetWindowPositionModeResponse::default()))
    }

    async fn get_window_position_mode(
        &self,
        _request: Request<RpcGetWindowPositionModeRequest>,
    ) -> Result<Response<RpcGetWindowPositionModeResponse>, Status> {
        let mode = self
            .server
            .get_window_position_mode()
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        let mode = match mode {
            WindowPositionMode::Static => "Static",
            WindowPositionMode::ActiveMonitor => "ActiveMonitor",
        };

        Ok(Response::new(RpcGetWindowPositionModeResponse {
            mode: mode.to_string(),
        }))
    }

    async fn download_plugin(
        &self,
        request: Request<RpcDownloadPluginRequest>,
    ) -> Result<Response<RpcDownloadPluginResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;

        let plugin_id = PluginId::from_string(plugin_id);

        self.server
            .download_plugin(plugin_id)
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcDownloadPluginResponse::default()))
    }

    async fn download_status(
        &self,
        _: Request<RpcDownloadStatusRequest>,
    ) -> Result<Response<RpcDownloadStatusResponse>, Status> {
        let status_per_plugin = self
            .server
            .download_status()
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?
            .into_iter()
            .map(|(plugin_id, status)| {
                let (status, message) = match status {
                    DownloadStatus::InProgress => (RpcDownloadStatus::InProgress, "".to_owned()),
                    DownloadStatus::Done => (RpcDownloadStatus::Done, "".to_owned()),
                    DownloadStatus::Failed { message } => (RpcDownloadStatus::Failed, message),
                };

                (
                    plugin_id.to_string(),
                    RpcDownloadStatusValue {
                        status: status.into(),
                        message,
                    },
                )
            })
            .collect();

        let response = RpcDownloadStatusResponse { status_per_plugin };

        Ok(Response::new(response))
    }

    async fn remove_plugin(
        &self,
        request: Request<RpcRemovePluginRequest>,
    ) -> Result<Response<RpcRemovePluginResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;

        let plugin_id = PluginId::from_string(plugin_id);

        self.server
            .remove_plugin(plugin_id)
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcRemovePluginResponse::default()))
    }

    async fn save_local_plugin(
        &self,
        request: Request<RpcSaveLocalPluginRequest>,
    ) -> Result<Response<RpcSaveLocalPluginResponse>, Status> {
        let request = request.into_inner();
        let path = request.path;

        let local_save_data = self
            .server
            .save_local_plugin(path)
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcSaveLocalPluginResponse {
            stdout_file_path: local_save_data.stdout_file_path,
            stderr_file_path: local_save_data.stderr_file_path,
        }))
    }
}
