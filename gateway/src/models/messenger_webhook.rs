use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MessengerVerifysubscriptionParam {
    #[serde(alias = "hub.mode")]
    pub hub_mode: Option<String>,
    #[serde(alias = "hub.verify_token")]
    pub hub_verify_token: Option<String>,
    #[serde(alias = "hub.challenge")]
    pub hub_challenge: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
pub struct Sender {
    id: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct QuickReplyPayload {
    payload: String,
}

#[allow(dead_code)]
impl QuickReplyPayload {
    pub fn get_payload(&self) -> &String {
        &self.payload
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Message {
    text: Option<String>,
    quick_reply: Option<QuickReplyPayload>,
}

#[allow(dead_code)]
impl Message {
    pub fn get_text(&self) -> String {
        self.text.clone().unwrap_or_default()
    }

    pub fn get_quick_reply(&self) -> Option<QuickReplyPayload> {
        self.quick_reply.clone()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Postback {
    payload: String,
}

#[allow(dead_code)]
impl MessagePostback {
    pub fn get_payload(&self) -> &String {
        &self.payload
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
pub struct Messaging {
    sender: Sender,
    postback: Option<MessagePostback>,
    message: Option<Message>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Entry {
    messaging: Vec<Messaging>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct InComingData {
    object: String,
    entry: Vec<Entry>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct MessagePostback {
    payload: String,
}

#[allow(dead_code)]
impl Postback {
    pub fn get_payload(&self) -> &String {
        &self.payload
    }
}
