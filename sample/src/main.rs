use neovim_lib::{Neovim, NeovimApi, Session, Value};
mod oauth;

enum Messages {
    Test,
    OAuth,
    Unknown(String),
}

impl From<String> for Messages {
    fn from(event: String) -> Self {
        match &event[..] {
            "test" => Messages::Test,
            "oauth" => Messages::OAuth,
            _ => Messages::Unknown(event),
        }
    }
}

struct EventHandler {
    nvim: Neovim,
}

impl EventHandler {
    fn new() -> EventHandler {
        let session = Session::new_parent().unwrap();
        let nvim = Neovim::new(session);
        EventHandler { nvim }
    }

    fn recv(&mut self) {
        let receiver = self.nvim.session.start_event_loop_channel();
        let _ = self.nvim.command("echo \"sup\"");

        for (event, values) in receiver {
            match Messages::from(event) {
                Messages::Test => {
                    self.nvim.command("echo \"testing\"").unwrap();
                }
                Messages::Unknown(event) => {
                    self.nvim
                        .command(&format!(
                            "echoerr \"Unknown command: {}. We support Init, Join, Send, OAuth\"",
                            event
                        ))
                        .unwrap();
                }
                Messages::OAuth => {
                    let mut args = values.iter();
                    let nickname = args.next().unwrap().to_string();
                    let client_id = args.next().unwrap().to_string();
                    let oauth_port = args.next().unwrap_or(&Value::from("6969")).to_string();

                    // This is blocking
                    let result = oauth::local_connect(client_id.clone(), oauth_port.clone());

                    match result {
                        Ok(access_token) => {
                            println!("token: {access_token}");
                            // le access_token = Option::from(access_token);

                            // let mut config = twitch_irc::ClientConfig::default();
                            // config.login_credentials = twitch_irc::login::StaticLoginCredentials {
                            //     credentials: twitch_irc::login::CredentialsPair {
                            //         login: "".to_string(),
                            //         token: self.twitch.access_token.clone(),
                            //     },
                            // };
                            // let (mut incoming_messages, client) =
                            //     twitch_irc::TwitchIRCClient::<
                            //         twitch_irc::SecureTCPTransport,
                            //         twitch_irc::login::StaticLoginCredentials,
                            //     >::new(config);
                            //
                            // self.client = Option::from(client);
                            // // self.incoming_messages = Option::from(incoming_messages);
                            //
                            // // TODO(Buser): Kick off listener????
                            // let join_handle = tokio::spawn(async move {
                            //     while let Some(message) = incoming_messages.recv().await {
                            //         println!("Received message: {:?}", message);
                            //     }
                            // });
                            //
                            // join_handles.push(join_handle);

                            self.nvim
                                .command(&format!("echo \"Connected to Twitch: {access_token}\""))
                                .unwrap();
                        }
                        Err(e) => {
                            self.nvim.command(&format!("echoerr \"{e}\"")).unwrap();
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let mut evt_hdl = EventHandler::new();

    evt_hdl.recv();
}
