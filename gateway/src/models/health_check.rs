use serde::{Deserialize, Serialize};

pub struct HealthCheck {
  pub date_now: String,
  pub ping: String,
}

impl HealthCheck {
  pub fn new(date_now: String, ping: String) -> Self {
    Self {
      date_now: date_now,
      ping: ping,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealtCheckResponse {
  pub date_now: String,
  pub ping: String,
}

impl From<HealthCheck> for HealtCheckResponse {
  fn from(healthcheck: HealthCheck) -> Self {
    Self {
      date_now: healthcheck.date_now,
      ping: healthcheck.ping
    }
  }
}
