use lapin::types::Boolean;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantConfig {
    pub channel_id: i32,
    pub app_id: i32,
    pub topic: String,
    pub enabled: Boolean,
    pub token: String,
}

impl MerchantConfig {
    pub fn new(channel_id: i32, app_id: i32, topic: String, enabled:Boolean, token: String) -> Self {
        Self {
            channel_id,
            app_id,
            topic,
            enabled,
            token
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantConfigResponse {
    pub channel_id: i32,
    pub app_id: i32,
    pub topic: String,
    pub enabled: Boolean,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantConfigWithTokenResponse {
    pub channel_id: i32,
    pub app_id: i32,
    pub topic: String,
    pub enabled: Boolean,
    pub token: String,
}


impl From<MerchantConfig> for MerchantConfigResponse {
    fn from(c: MerchantConfig) -> Self {
        Self {
            channel_id: c.channel_id,
            app_id: c.app_id,
            topic: c.topic,
            enabled: c.enabled,
        }
    }
}

impl From<MerchantConfig> for MerchantConfigWithTokenResponse {
    fn from(c: MerchantConfig) -> Self {
        Self {
            channel_id: c.channel_id,
            app_id: c.app_id,
            topic: c.topic,
            enabled: c.enabled,
            token: c.token,
        }
    }
}