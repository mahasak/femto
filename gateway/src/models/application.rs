use serde::{Deserialize, Serialize};

pub struct Application {
  pub app_id: String,
  pub app_name: String,
  pub topic: String,
  pub enabled: bool,
}

impl Application {
  pub fn new(app_id: String, app_name: String, topic: String, enabled:bool) -> Self {
    Self {
      app_id: app_id,
      app_name: app_name,
      topic: topic,
      enabled: enabled
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationResponse {
  pub app_id: String,
  pub app_name: String,
  pub topic: String,
  pub enabled: bool,
}

impl From<Application> for ApplicationResponse {
  fn from(app: Application) -> Self {
    Self {
      app_id: app.app_id,
      app_name: app.app_name,
      topic: app.topic,
      enabled: app.enabled
    }
  }
}