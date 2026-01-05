//! AdiOS ecosystem integration for this plugin
//! 
//! This module handles communication with AdiOS core and other plugins.

use adios_core::plugin::{Plugin, PluginInfo, PluginResult, PluginCategory};
use adios_core::context::AppContext;
use adios_core::events::EventBus;
use async_trait::async_trait;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Plugin state for ecosystem integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    pub initialized: bool,
    pub active: bool,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning(String),
    Error(String),
}

/// Main plugin structure for AdiOS integration
pub struct AdiosPlugin {
    info: PluginInfo,
    state: PluginState,
    ctx: Option<Arc<AppContext>>,
    bus: Option<Arc<EventBus>>,
}

impl AdiosPlugin {
    pub fn new(info: PluginInfo) -> Self {
        Self {
            info,
            state: PluginState {
                initialized: false,
                active: false,
                last_sync: None,
                health_status: HealthStatus::Healthy,
            },
            ctx: None,
            bus: None,
        }
    }
    
    /// Get current plugin state
    pub fn state(&self) -> &PluginState {
        &self.state
    }
    
    /// Update health status
    pub fn update_health(&mut self, status: HealthStatus) {
        self.state.health_status = status;
    }
    
    /// Sync with ecosystem
    pub async fn sync_with_ecosystem(&mut self) -> PluginResult<()> {
        if let Some(bus) = &self.bus {
            // Publish sync event
            bus.publish(adios_core::events::Event::PluginSync {
                plugin_id: self.info.id.clone(),
                timestamp: chrono::Utc::now(),
            });
            
            self.state.last_sync = Some(chrono::Utc::now());
        }
        
        Ok(())
    }
    
    /// Handle inter-plugin communication
    pub async fn handle_plugin_message(&self, from: &str, message: serde_json::Value) -> PluginResult<serde_json::Value> {
        // Default implementation - override in specific plugins
        tracing::info!(from = %from, "Received plugin message");
        Ok(serde_json::json!({"status": "received"}))
    }
    
    /// Register with plugin discovery service
    pub async fn register_capabilities(&self) -> PluginResult<()> {
        if let Some(ctx) = &self.ctx {
            // Register plugin capabilities with discovery service
            tracing::info!(plugin_id = %self.info.id, "Registering plugin capabilities");
        }
        
        Ok(())
    }
}

#[async_trait]
impl Plugin for AdiosPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }
    
    async fn init(&mut self, ctx: Arc<AppContext>, bus: Arc<EventBus>) -> PluginResult<()> {
        tracing::info!(plugin_id = %self.info.id, "Initializing plugin");
        
        self.ctx = Some(ctx);
        self.bus = Some(bus);
        self.state.initialized = true;
        
        // Register capabilities
        self.register_capabilities().await?;
        
        Ok(())
    }
    
    async fn start(&mut self) -> PluginResult<()> {
        tracing::info!(plugin_id = %self.info.id, "Starting plugin");
        
        self.state.active = true;
        
        // Sync with ecosystem on start
        self.sync_with_ecosystem().await?;
        
        Ok(())
    }
    
    async fn stop(&mut self) -> PluginResult<()> {
        tracing::info!(plugin_id = %self.info.id, "Stopping plugin");
        
        self.state.active = false;
        
        Ok(())
    }
    
    async fn tick(&mut self) -> PluginResult<()> {
        // Periodic health check and sync
        if self.state.active {
            // Check if we need to sync (every 5 minutes)
            if let Some(last_sync) = self.state.last_sync {
                let now = chrono::Utc::now();
                if now.signed_duration_since(last_sync).num_minutes() >= 5 {
                    self.sync_with_ecosystem().await?;
                }
            }
        }
        
        Ok(())
    }
    
    fn is_healthy(&self) -> bool {
        matches!(self.state.health_status, HealthStatus::Healthy)
    }
    
    fn status(&self) -> String {
        match &self.state.health_status {
            HealthStatus::Healthy => "Healthy".to_string(),
            HealthStatus::Warning(msg) => format!("Warning: {}", msg),
            HealthStatus::Error(msg) => format!("Error: {}", msg),
        }
    }
}

/// Helper function to create plugin info from manifest
pub fn create_plugin_info_from_manifest() -> PluginResult<PluginInfo> {
    // Read adios-plugin.toml
    let manifest_content = std::fs::read_to_string("adios-plugin.toml")
        .map_err(|e| adios_core::plugin::PluginError::InitError(format!("Failed to read manifest: {}", e)))?;
    
    let manifest: toml::Value = toml::from_str(&manifest_content)
        .map_err(|e| adios_core::plugin::PluginError::InitError(format!("Failed to parse manifest: {}", e)))?;
    
    let plugin_section = manifest.get("plugin")
        .ok_or_else(|| adios_core::plugin::PluginError::InitError("No [plugin] section in manifest".to_string()))?;
    
    let info = adios_core::plugin::PluginInfoBuilder::new(
        plugin_section.get("name").and_then(|v| v.as_str()).unwrap_or("unknown"),
        plugin_section.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown Plugin")
    )
    .version(plugin_section.get("version").and_then(|v| v.as_str()).unwrap_or("0.1.0"))
    .description(plugin_section.get("description").and_then(|v| v.as_str()).unwrap_or(""))
    .author(plugin_section.get("author").and_then(|v| v.as_str()).unwrap_or(""))
    .category(match plugin_section.get("category").and_then(|v| v.as_str()).unwrap_or("other") {
        "productivity" => PluginCategory::Productivity,
        "development" => PluginCategory::Core,
        "enterprise" => PluginCategory::Integration,
        "communication" => PluginCategory::Communication,
        "wellness" => PluginCategory::Wellness,
        "display" => PluginCategory::Display,
        _ => PluginCategory::Other,
    })
    .build();
    
    Ok(info)
}
