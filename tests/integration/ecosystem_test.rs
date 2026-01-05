//! Integration tests for AdiOS ecosystem

use std::sync::Arc;

#[tokio::test]
async fn test_plugin_registration() {
    // Test plugin can be registered with AdiOS core
    let plugin = super::super::create_plugin().unwrap();
    assert!(!plugin.info().id.is_empty());
    assert!(!plugin.info().name.is_empty());
}

#[tokio::test]
async fn test_plugin_lifecycle() {
    // Test complete plugin lifecycle
    let mut plugin = super::super::create_plugin().unwrap();
    
    // Mock context and event bus
    let ctx = Arc::new(adios_core::context::AppContext::new("test".to_string(), "test".to_string()));
    let bus = Arc::new(adios_core::events::EventBus::new());
    
    // Test initialization
    plugin.init(ctx, bus).await.unwrap();
    
    // Test start
    plugin.start().await.unwrap();
    
    // Test health check
    assert!(plugin.is_healthy());
    
    // Test stop
    plugin.stop().await.unwrap();
}

#[tokio::test]
async fn test_ecosystem_communication() {
    // Test inter-plugin communication
    // This would be expanded based on specific plugin needs
}
