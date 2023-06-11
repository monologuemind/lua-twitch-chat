use neovim_lib::{Neovim, NeovimApi};
use std::sync::Arc;
use tokio::sync::RwLock;

mod browser;

pub async fn oauth(
    client_id: &Option<String>,
    nickname: &Option<String>,
    oauth_port: &String,
    nvim: &mut Neovim,
    incoming_messages_arc: &Arc<
        tokio::sync::Mutex<
            tokio::sync::mpsc::UnboundedReceiver<twitch_irc::message::ServerMessage>,
        >,
    >,
    client_arc: &Arc<
        RwLock<
            twitch_irc::TwitchIRCClient<
                twitch_irc::transport::tcp::TCPTransport<twitch_irc::transport::tcp::TLS>,
                twitch_irc::login::StaticLoginCredentials,
            >,
        >,
    >,
) -> Option<String> {
    // This is blocking
    let result = browser::local_connect(client_id.clone().unwrap(), oauth_port.clone(), nvim);

    match result {
        Ok(access_token) => {
            nvim.command(&format!("echo \"access_token: {access_token}\""))
                .unwrap();

            let mut config = twitch_irc::ClientConfig::default();
            config.login_credentials = twitch_irc::login::StaticLoginCredentials {
                credentials: twitch_irc::login::CredentialsPair {
                    login: nickname.clone().unwrap(),
                    token: Option::from(access_token.clone()),
                },
            };

            let updated_twitch_client = twitch_irc::TwitchIRCClient::<
                twitch_irc::SecureTCPTransport,
                twitch_irc::login::StaticLoginCredentials,
            >::new(config);

            let mut incoming_messages = incoming_messages_arc.lock().await;
            *incoming_messages = updated_twitch_client.0;

            let mut client = client_arc.write().await;
            *client = updated_twitch_client.1;

            nvim.command(&format!("echo \"Connected to Twitch\""))
                .unwrap();

            return Option::from(access_token);
        }
        Err(e) => {
            nvim.command(&format!("echo \"Error authing: {e}\""))
                .unwrap();
            return None;
        }
    }
}
