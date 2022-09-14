use serde::Deserialize;
pub use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, SendError, Sender, TryRecvError};
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;
use thiserror::Error;
use websocket::header::Headers;
use websocket::receiver::Reader;
use websocket::sender::Writer;
use websocket::Message;

const OS: &str = "Browser";
const DEVICE: &str = "Mozilla/5.0 (Linux; Android 8.0.0; SM-G955U Build/R16NW) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Mobile Safari/537.36";

#[derive(Error, Debug)]
pub enum FlipperError {
    #[error("parse websocket url error")]
    ParseUrl(#[from] websocket::client::ParseError),
    #[error("connect websocket error")]
    ConnectWebSocket(#[from] websocket::WebSocketError),
    #[error("split websocket error")]
    SplitWebSocket(#[from] std::io::Error),
    #[error("deserialize json error")]
    DeserializeMessage(#[from] serde_json::Error),
    #[error("unexpected message")]
    UnexpectedMessage,
    #[error("json field `{0}` not exist")]
    GetJsonField(String),
    #[error("json field `{0}` is not str")]
    CastJsonFieldToStr(String),
    #[error("send to channel")]
    SendToChannel(#[from] SendError<RecvdMessage>),
    #[error("channel disconnected")]
    ChannelDisconnected,
}

pub type Result<T> = std::result::Result<T, FlipperError>;

pub trait FlipperPlugin {
    fn get_id(&self) -> String;
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);
    fn run_in_background(&self) -> bool;
    fn call(&mut self, method: &str, params: &Value) -> Result<Value>;
    fn is_method_supported(&self, method: &str) -> bool;
}

#[derive(Debug, Deserialize)]
pub struct RecvdMessage {
    method: String,
    #[serde(default)]
    id: usize,
    #[serde(default)]
    params: Value,
}

pub struct FlipperClient {
    writer: Writer<TcpStream>,
    reader: Option<Reader<TcpStream>>,
    plugins: HashMap<String, Box<dyn FlipperPlugin>>,
    receiver: Receiver<RecvdMessage>,
    sender: Option<Sender<RecvdMessage>>,
}

impl FlipperClient {
    pub fn connect(app_name: &str, device_id: &str) -> Result<Self> {
        let uri = format!(
            "ws://localhost:8333?os={OS}&device={DEVICE}&device_id={device_id}&app={app_name}"
        );

        let mut headers = Headers::new();
        headers.set(websocket::header::Origin(
            "http://localhost:8333".to_string(),
        ));
        let client = websocket::ClientBuilder::new(&uri)?
            .custom_headers(&headers)
            .connect_insecure()?;
        let (reader, writer) = client.split()?;
        let (sender, receiver) = channel();
        Ok(Self {
            reader: Some(reader),
            writer,
            sender: Some(sender),
            receiver,
            plugins: HashMap::new(),
        })
    }

    pub fn add_plugin(&mut self, plugin: impl FlipperPlugin + 'static) -> Result<()> {
        self.plugins.insert(plugin.get_id(), Box::new(plugin));
        self.refresh_plugins()
    }

    fn get_plugin_mut<'a>(&'a mut self, id: &str) -> Option<&'a mut dyn FlipperPlugin> {
        Some(self.plugins.get_mut(id)?.as_mut())
    }

    fn refresh_plugins(&mut self) -> Result<()> {
        self.send_json(json!({
            "method": "refreshPlugins",
        }))
    }

    fn send_str(&mut self, data: impl AsRef<str>) -> Result<()> {
        self.writer.send_message(&Message::text(data.as_ref()))?;
        Ok(())
    }

    fn response_success(&mut self, id: usize, data: Value) -> Result<()> {
        self.send_json(json!({
            "id": id,
            "success": data,
        }))
    }

    fn send_json(&mut self, data: Value) -> Result<()> {
        self.send_str(data.to_string())?;
        Ok(())
    }

    pub fn spawn_recving_data_thread(&mut self, mut noti: impl FnMut() + Send + 'static) {
        let mut reader = self.reader.take().expect("reader is none");
        let sender = self.sender.take().expect("sender is none");

        fn pipe_msg(
            reader: &mut Reader<TcpStream>,
            sender: &Sender<RecvdMessage>,
            noti: &mut dyn FnMut(),
        ) -> Result<()> {
            let msg = reader.recv_message()?;
            let text = match msg {
                websocket::OwnedMessage::Text(text) => text,
                _ => {
                    return Err(FlipperError::UnexpectedMessage);
                }
            };
            let data: RecvdMessage = serde_json::from_str(&text)?;
            sender.send(data)?;
            noti();
            Ok(())
        }

        let _ = thread::spawn(move || loop {
            if let Err(e) = pipe_msg(&mut reader, &sender, &mut noti) {
                tracing::error!("pipe msg error: {:?}", e);
                sleep(Duration::from_secs(1));
            }
        });
    }

    pub fn dispatch_event(&mut self) -> Result<()> {
        loop {
            let data = self.receiver.try_recv();
            let data = match data {
                Ok(d) => d,
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => return Err(FlipperError::ChannelDisconnected),
            };

            tracing::debug!(req = ?&data);

            let ret = match data.method.as_str() {
                "getPlugins" => self.get_plugins()?,
                "getBackgroundPlugins" => self.get_background_plugins()?,
                "init" => self.init(&data.params)?,
                "deinit" => self.deinit(&data.params)?,
                "execute" => self.execute(&data.params)?,
                "isMethodSupported" => self.is_method_supported(&data.params)?,
                _ => Value::Null,
            };

            tracing::debug!(resp = ?&ret);
            self.response_success(data.id, ret)?;
        }
    }

    fn get_plugins(&mut self) -> Result<Value> {
        let plugins = self.plugins.keys().collect::<Vec<_>>();
        Ok(json!({
            "plugins": plugins,
        }))
    }

    fn get_background_plugins(&self) -> Result<Value> {
        Ok(json!({
            "plugins": []
        }))
    }

    fn connect_plugin(&mut self, plugin_id: &str) {
        if let Some(plugin) = self.get_plugin_mut(plugin_id) {
            plugin.on_connect();
        }
    }

    fn disconnect_plugin(&mut self, plugin_id: &str) {
        if let Some(mut plugin) = self.plugins.remove(plugin_id) {
            plugin.on_disconnect();
        }
    }

    fn execute(&mut self, params: &Value) -> Result<Value> {
        let ident = get_attr_str(params, "api")?;
        if let Some(plugin) = self.get_plugin_mut(ident) {
            let method = get_attr_str(params, "method")?;
            let params = params.get("params").unwrap_or(&Value::Null);
            let ret = plugin.call(method, params)?;
            tracing::debug!(?method, ?params, ?ret);
            return Ok(ret);
        }

        Ok(Value::Null)
    }

    fn init(&mut self, params: &Value) -> Result<Value> {
        let identifier = get_attr_str(params, "plugin")?;
        self.connect_plugin(identifier);
        Ok(Value::Null)
    }

    fn deinit(&mut self, params: &Value) -> Result<Value> {
        let identifier = get_attr_str(params, "plugin")?;
        self.disconnect_plugin(identifier);
        Ok(Value::Null)
    }

    fn is_method_supported(&mut self, params: &Value) -> Result<Value> {
        let ident = get_attr_str(params, "api")?;
        let supported = if let Some(plugin) = self.get_plugin_mut(ident) {
            let method = get_attr_str(params, "method")?;
            let ret = plugin.is_method_supported(method);
            tracing::debug!(?method, ?ret);
            ret
        } else {
            false
        };

        Ok(json!({ "isSupported": supported }))
    }
}

fn get_attr_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .ok_or_else(|| FlipperError::GetJsonField(key.to_string()))?
        .as_str()
        .ok_or_else(|| FlipperError::CastJsonFieldToStr(key.to_string()))
}

impl Drop for FlipperClient {
    fn drop(&mut self) {
        for (_, mut plugin) in self.plugins.drain() {
            plugin.on_disconnect();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TicTacToePlugin {}

    impl FlipperPlugin for TicTacToePlugin {
        fn get_id(&self) -> String {
            "ReactNativeTicTacToe".to_string()
        }

        fn on_connect(&mut self) {
            tracing::debug!("connected");
        }

        fn on_disconnect(&mut self) {
            tracing::debug!("disconnected");
        }

        fn run_in_background(&self) -> bool {
            false
        }

        fn call(&mut self, method: &str, params: &Value) -> Result<Value> {
            Ok(Value::Null)
        }

        fn is_method_supported(&self, method: &str) -> bool {
            false
        }
    }

    #[test]
    fn test_tic_tok() -> Result<()> {
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        let mut flipper = FlipperClient::connect("test-flipper", "flipper-1")?;
        flipper.add_plugin(Box::new(TicTacToePlugin {}))?;
        flipper.recv_data()?;
        Ok(())
    }
}
