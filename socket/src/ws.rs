use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use url::Url;

// TODO(Buser): Need to parse the twitch api/chat messages
pub fn parser() {}

pub async fn runner() {
    let url = Url::parse("ws://irc-ws.chat.twitch.tv:80").unwrap();
    let (mut ws_stream, _) = connect_async(url).await.unwrap();

    let _ = ws_stream
        .send(tokio_tungstenite::tungstenite::Message::Text(
            "PASS test".to_string(),
        ))
        .await;
    let _ = ws_stream
        .send(tokio_tungstenite::tungstenite::Message::Text(
            "NICK test".to_string(),
        ))
        .await;

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(parsed) => {
                println!("msg: {parsed:?}");
                if parsed.is_text() || parsed.is_binary() {
                    ws_stream.send(parsed).await.unwrap();
                }
            }
            Err(e) => {
                println!("Error from twitch");
                panic!("{e}");
            }
        }
    }
}
