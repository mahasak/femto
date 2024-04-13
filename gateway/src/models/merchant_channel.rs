use serde::{Deserialize, Serialize};

pub struct MerchantChannel {
  pub id: i32,
  pub ref_id: String,
  pub name: String,
  pub ref_type: String,
  pub token: String,
}

impl MerchantChannel {
  pub fn new(id: i32, ref_id: String, name: String, ref_type:String, token: String) -> Self {
    Self {
      id: id,
      ref_id: ref_id,
      name: name,
      ref_type: ref_type,
      token: token
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantChannelResponse {
  pub id: i32,
  pub ref_id: String,
  pub name: String,
  pub ref_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantChannelWithTokenResponse {
  pub id: i32,
  pub ref_id: String,
  pub name: String,
  pub ref_type: String,
  pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantChannelEligbleResponse {
  pub ref_id: String,
  pub eligible: bool,
}

impl From<MerchantChannel> for MerchantChannelResponse {
  fn from(c: MerchantChannel) -> Self {
    Self {
      id: c.id,
      ref_id: c.ref_id,
      name: c.name,
      ref_type: c.ref_type,
    }
  }
}

impl From<MerchantChannel> for MerchantChannelWithTokenResponse {
  fn from(c: MerchantChannel) -> Self {
    Self {
      id: c.id,
      ref_id: c.ref_id,
      name: c.name,
      ref_type: c.ref_type,
      token: c.token,
    }
  }
}