use emit::events::Event;
use emit::LogLevel;
use reqwest::header::HeaderValue;
use std::borrow::Cow;
use std::convert::Into;
use std::error::Error;
use std::fmt::Write;

pub const DEFAULT_EVENT_BODY_LIMIT_BYTES: usize = 1024 * 256;
pub const DEFAULT_BATCH_LIMIT_BYTES: usize = 1024 * 1024 * 10;
pub const LOCAL_SERVER_URL: &'static str = "http://localhost:5341/";

#[derive(Debug)]
pub struct SeqCollectorBuilder {
    api_key: Option<Cow<'static, str>>,
    event_body_limit_bytes: usize,
    batch_limit_bytes: usize,
    server_url: Cow<'static, str>,
}

impl SeqCollectorBuilder {
    pub fn new() -> Self {
        SeqCollectorBuilder {
            api_key: None,
            event_body_limit_bytes: DEFAULT_EVENT_BODY_LIMIT_BYTES,
            batch_limit_bytes: DEFAULT_BATCH_LIMIT_BYTES,
            server_url: LOCAL_SERVER_URL.into(),
        }
    }

    pub fn server_url<T: Into<String>>(mut self, server_url: T) -> Self {
        self.server_url = Cow::Owned(server_url.into());
        self
    }

    pub fn api_key<T: Into<String>>(mut self, api_key: T) -> Self {
        self.api_key = Some(Cow::Owned(api_key.into()));
        self
    }

    #[allow(dead_code)]
    pub fn event_body_limit_bytes(mut self, event_body_limit_bytes: usize) -> Self {
        self.event_body_limit_bytes = event_body_limit_bytes;
        self
    }

    #[allow(dead_code)]
    pub fn batch_limit_bytes(mut self, batch_limit_bytes: usize) -> Self {
        self.batch_limit_bytes = batch_limit_bytes;
        self
    }

    pub fn build(self) -> SeqCollector {
        SeqCollector {
            api_key: self.api_key.map(|k| k.into_owned()),
            event_body_limit_bytes: self.event_body_limit_bytes,
            batch_limit_bytes: self.batch_limit_bytes,
            endpoint: format!("{}api/events/raw/", self.server_url),
        }
    }
}

// 0 is "OFF", but fatal is the best effort for rendering this if we ever get an
// event with that level.
static SEQ_LEVEL_NAMES: [&'static str; 6] = [
    "Fatal",
    "Error",
    "Warning",
    "Information",
    "Debug",
    "Verbose",
];

#[derive(Debug)]
pub struct SeqCollector {
    api_key: Option<String>,
    event_body_limit_bytes: usize,
    batch_limit_bytes: usize,
    endpoint: String,
}

impl SeqCollector {
    #[allow(dead_code)]
    pub fn new<T: Into<String>>(server_url: T) -> SeqCollector {
        Self::builder().server_url(server_url).build()
    }

    #[allow(dead_code)]
    pub fn new_local() -> SeqCollector {
        Self::builder().build()
    }

    pub fn builder() -> SeqCollectorBuilder {
        SeqCollectorBuilder::new()
    }

    fn send_batch(&self, payload: &String) -> Result<(), Box<dyn Error>> {
        log::debug!("logging {}", payload);
        let client = reqwest::blocking::Client::new();
        if !self.api_key.is_none() {
            let api_key = self.api_key.clone().unwrap();
            let _res = client
                .post(&self.endpoint)
                .body(payload.clone())
                .header("X-Seq-ApiKey", HeaderValue::from_str(&api_key).unwrap())
                .send()?;
        } else {
            let _res = client.post(&self.endpoint).body(payload.clone()).send()?;
        }

        Ok(())
    }
}

const HEADER: &'static str = "{\"Events\":[";
const HEADER_LEN: usize = 11;
const FOOTER: &'static str = "]}";
const FOOTER_LEN: usize = 2;

impl emit::collectors::AcceptEvents for SeqCollector {
    fn accept_events(&self, events: &[Event<'static>]) -> Result<(), Box<dyn Error>> {
        let mut next = HEADER.to_owned();
        let mut count = HEADER_LEN + FOOTER_LEN;
        let mut delim = "";

        for event in events {
            let mut payload = format_payload(event);
            if payload.len() > self.event_body_limit_bytes {
                payload = format_oversize_placeholder(event);
                if payload.len() > self.event_body_limit_bytes {
                    // TODO - self-log
                    // error!("An oversize event was detected but the size limit is so low a placeholder cannot be substituted");
                    continue;
                }
            }

            // Make sure at least one event is included in each batch
            if delim != "" && count + delim.len() + payload.len() > self.batch_limit_bytes {
                write!(next, "{}", FOOTER).unwrap();
                let _ = self.send_batch(&next);

                next = format!("{}{}", HEADER, payload);
                count = HEADER_LEN + FOOTER_LEN + payload.len();
                delim = ",";
            } else {
                write!(next, "{}{}", delim, payload).unwrap();
                count += delim.len() + payload.len();
                delim = ",";
            }
        }

        write!(next, "{}", FOOTER).unwrap();
        let _ = self.send_batch(&next);

        Ok(())
    }
}

fn format_payload(event: &Event) -> String {
    let mut body = format!(
        "{{\"Timestamp\":\"{}\",\"Level\":\"{}\",\"MessageTemplate\":{},\"Properties\":{{",
        event.timestamp().format("%FT%TZ"),
        to_seq_level(event.level()),
        serde_json::to_string(event.message_template().text()).unwrap()
    );

    let mut first = true;
    for (n, v) in event.properties() {
        if !first {
            body.push_str(",");
        } else {
            first = false;
        }

        write!(&mut body, "\"{}\":{}", n, v.to_json()).unwrap();
    }

    body.push_str("}}");
    body
}

fn format_oversize_placeholder(event: &Event) -> String {
    let initial: String = if event.message_template().text().len() > 64 {
        event
            .message_template()
            .text()
            .chars()
            .take(64)
            .into_iter()
            .collect()
    } else {
        event.message_template().text().clone()
    };

    format!("{{\"Timestamp\":\"{}\",\"Level\":\"{}\",\"MessageTemplate\":\"(Event too large) {{initial}}...\",\"Properties\":{{\"target\":\"emit::collectors::seq\",\"initial\":{}}}}}",
        event.timestamp().format("%FT%TZ"),
        to_seq_level(event.level()),
        serde_json::to_string(&initial).unwrap())
}

fn to_seq_level(level: LogLevel) -> &'static str {
    SEQ_LEVEL_NAMES[level as usize]
}
