use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Serialize, Deserialize, Debug)]
pub struct MessengerVerifysubscription {
    #[serde(alias = "hub.mode")]
    pub hub_mode: Option<String>,
    #[serde(alias = "hub.verify_token")]
    pub hub_verify_token: Option<String>,
    #[serde(alias = "hub.challenge")]
    pub hub_challenge: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize)]
pub struct Sender {
    id: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize)]
pub struct Receipient {
    id: String,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Default, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct DeliveryInfo {
    mids: Vec<String>,
    watermark: Number,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadInfo {
    watermark: Number,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLinkingInfo {
    status: String,
    authorization_code: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Messaging {
    sender: Sender,
    postback: Option<MessagePostback>,
    message: Option<Message>,
    delivery: Option<DeliveryInfo>,
    read: Option<ReadInfo>,
    account_linking: Option<AccountLinkingInfo>,
    recipient: Receipient,
    reaction: Option<String>,
    timestamp: Number,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookEntry {
    id: String,
    time: Number,
    messaging: Option<Vec<Messaging>>,
    #[serde(flatten)]
    changes: Option<Vec<ChangesEvent>>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangesEvent {
    field: String,
    value: Option<ChangeEventValue>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeEventValue {
    page_id: String,            //Generic field
    invoice_id: Option<String>, // P2M invoice field
    media_id: Option<String>,   // P2M Bankslip field
    buyer_id: Option<String>,   // P2M Bankslip field
    timestamp: Number,
    event: Option<String>,
    payment: Option<PaymentInfo>, // P2M Bankslip field
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentAmount {
    amount: String,
    currency: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentInfo {
    payment_amount: String,
    payment_method: String,
    creation_time: Number,
    buyer_id: String,
    order_id: Option<String>,
    payment_id: String,
    metadata: Option<PaymentMetadata>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentMetadata {
    image_url: Option<String>,
    bank_transfer_id: Option<String>,
    media_id: Option<String>,
    amount_validated: Option<PaymentAmount>,
    transaction_time: Option<Number>,
    validation_info: Option<BankSlipValidationInfo>,
    validation_status: Option<String>,
    receiver_name: Option<String>,
    receiver_bank_account_id: Option<String>,
    receiver_bank_code: Option<String>,
    sender_name: Option<String>,
    sender_bank_account_id: Option<String>,
    sender_bank_code: Option<String>,
    hpp_payment_link: Option<HppMetadata>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct HppMetadata {
    psp_txn_id: String,
    payment_status: String,
    payment_provider: String,
    updated_time: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct BankSlipValidationInfo {
    payment_amount: PaymentAmount,
    payment_time: String,
    is_seller_onboarded: bool,
    matches_seller_account: bool,
    is_duplicate: bool,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct WrappedMessage {
    trace_id: String,
    page_entry: WebhookEntry,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct MessengerWebhook {
    object: String,
    entry: Vec<WebhookEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct MessagePostback {
    payload: String,
}

#[allow(dead_code)]
impl Postback {
    pub fn get_payload(&self) -> &String {
        &self.payload
    }
}
