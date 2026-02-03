mod binary_manager;
mod logger;
mod simple_temp_dir;

use binary_manager::BinaryManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zed_extension_api::serde_json::Value;
use zed_extension_api::{
    self as zed, serde_json, DebugAdapterBinary, DebugConfig, DebugRequest, DebugScenario,
    DebugTaskDefinition, StartDebuggingRequestArguments, StartDebuggingRequestArgumentsRequest,
    Worktree,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetCoreDbgDebugConfig {
    pub request: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_at_entry: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_id: Option<ProcessId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub just_my_code: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_step_filtering: Option<bool>,
}

/// Represents a process id that can be either an integer or a string (containing a number)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ProcessId {
    Int(i32),
    String(String),
}

#[derive(Default)]
struct NetCoreDbgExtension {
    binary_manager: BinaryManager,
}

impl NetCoreDbgExtension {
    const ADAPTER_NAME: &str = "netcoredbg";
}

impl zed::Extension for NetCoreDbgExtension {
    fn new() -> Self {
        Self::default()
    }

    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        config: DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        if adapter_name != Self::ADAPTER_NAME {
            return Err(format!("Cannot create binary for adapter: {adapter_name}"));
        }

        let configuration = config.config.to_string();
        let parsed_config: NetCoreDbgDebugConfig =
            serde_json::from_str(&configuration).map_err(|e| {
                format!("Failed to parse debug configuration: {}. Expected NetCoreDbg configuration format.", e)
            })?;

        let request = match parsed_config.request.as_str() {
            "launch" => StartDebuggingRequestArgumentsRequest::Launch,
            "attach" => StartDebuggingRequestArgumentsRequest::Attach,
            other => {
                return Err(format!(
                    "Invalid 'request' value: '{}'. Expected 'launch' or 'attach'",
                    other
                ))
            }
        };

        let binary_path = self
            .binary_manager
            .get_binary_path(user_provided_debug_adapter_path)?;

        Ok(DebugAdapterBinary {
            command: Some(binary_path),
            arguments: vec!["--interpreter=vscode".to_string()],
            envs: parsed_config.env.into_iter().collect(),
            cwd: Some(parsed_config.cwd.unwrap_or_else(|| worktree.root_path())),
            connection: None,
            request_args: StartDebuggingRequestArguments {
                configuration,
                request,
            },
        })
    }

    fn dap_request_kind(
        &mut self,
        adapter_name: String,
        config: Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        if adapter_name != Self::ADAPTER_NAME {
            return Err(format!("Unknown adapter: {}", adapter_name));
        }

        match config.get("request").and_then(|v| v.as_str()) {
            Some("launch") => Ok(StartDebuggingRequestArgumentsRequest::Launch),
            Some("attach") => Ok(StartDebuggingRequestArgumentsRequest::Attach),
            Some(other) => Err(format!(
                "Invalid 'request' value: '{}'. Expected 'launch' or 'attach'",
                other
            )),
            None => Err(
                "Debug configuration missing required 'request' field. Must be 'launch' or 'attach'"
                    .to_string(),
            ),
        }
    }

    fn dap_config_to_scenario(&mut self, config: DebugConfig) -> Result<DebugScenario, String> {
        match config.request {
            DebugRequest::Launch(launch) => {
                let adapter_config = NetCoreDbgDebugConfig {
                    request: "launch".to_string(),
                    program: Some(launch.program),
                    args: if launch.args.is_empty() {
                        None
                    } else {
                        Some(launch.args)
                    },
                    cwd: launch.cwd,
                    env: launch.envs.into_iter().collect(),
                    stop_at_entry: config.stop_on_entry,
                    process_id: None,
                    just_my_code: None,
                    enable_step_filtering: None,
                };

                let config_json = serde_json::to_string(&adapter_config)
                    .map_err(|e| format!("Failed to serialize launch config: {}", e))?;

                Ok(DebugScenario {
                    label: config.label,
                    adapter: config.adapter,
                    build: None,
                    config: config_json,
                    tcp_connection: None,
                })
            }
            DebugRequest::Attach(attach) => {
                let process_id = attach.process_id.ok_or_else(|| {
                    "Attach mode requires a process ID. Please select a process from the attach modal.".to_string()
                })?;

                let adapter_config = NetCoreDbgDebugConfig {
                    request: "attach".to_string(),
                    program: None,
                    args: None,
                    cwd: None,
                    env: HashMap::new(),
                    stop_at_entry: config.stop_on_entry,
                    process_id: Some(ProcessId::Int(process_id.try_into().map_err(|_| {
                        format!("Process ID {} is too large to fit in i32", process_id)
                    })?)),
                    just_my_code: None,
                    enable_step_filtering: None,
                };

                let config_json = serde_json::to_string(&adapter_config)
                    .map_err(|e| format!("Failed to serialize attach config: {}", e))?;

                Ok(DebugScenario {
                    label: config.label,
                    adapter: config.adapter,
                    build: None,
                    config: config_json,
                    tcp_connection: None,
                })
            }
        }
    }
}

zed::register_extension!(NetCoreDbgExtension);
