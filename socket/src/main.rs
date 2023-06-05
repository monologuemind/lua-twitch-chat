use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use neovim_lib::neovim_api::Buffer;
use neovim_lib::{Neovim, NeovimApi, Session, Value};

mod oauth;

// This is stolen from the session.rs of neovim_lib. It is not exposed
macro_rules! call_args {
    () => (Vec::new());
    ($($e:expr), +,) => (call_args![$($e),*]);
    ($($e:expr), +) => {{
        let mut vec = Vec::new();
        $(
            vec.push($e.into_val());
        )*
        vec
    }};
}

enum Messages {
    Test,
    Init,
    Join,
    Send,
    OAuth,
    // Exit,
    Unknown(String),
}

impl From<String> for Messages {
    fn from(event: String) -> Self {
        match &event[..] {
            "test" => Messages::Test,
            "init" => Messages::Init,
            "join" => Messages::Join,
            // "exit" => Messages::Exit,
            "send" => Messages::Send,
            "oauth" => Messages::OAuth,
            _ => Messages::Unknown(event),
        }
    }
}

struct Twitch {
    nickname: Option<String>,
    // jzy5ssncfqreqxewn978xmgw03jy5w
    client_id: Option<String>,
    access_token: Option<String>,
}

impl Twitch {
    fn init() -> Twitch {
        Twitch {
            nickname: None,
            client_id: None,
            access_token: None,
        }
    }
}

struct EventHandler {
    nvim: Neovim,
    twitch: Twitch,
    oauth_port: String,
    client: Option<
        twitch_irc::TwitchIRCClient<
            twitch_irc::SecureTCPTransport,
            twitch_irc::login::StaticLoginCredentials,
        >,
    >,
    listening: bool,
    // end: bool,
}

impl EventHandler {
    fn new() -> EventHandler {
        let session = Session::new_parent().unwrap();
        let nvim = Neovim::new(session);
        let twitch = Twitch::init();

        EventHandler {
            nvim,
            twitch,
            oauth_port: String::from("6969"),
            client: None,
            listening: false,
            // end: false,
        }
    }

    async fn recv(
        &mut self,
        join_handles: &mut Vec<tokio::task::JoinHandle<()>>,
        incoming_messages_arc: Arc<
            Mutex<tokio::sync::mpsc::UnboundedReceiver<twitch_irc::message::ServerMessage>>,
        >,
        client_arc: Arc<
            Mutex<
                twitch_irc::TwitchIRCClient<
                    twitch_irc::transport::tcp::TCPTransport<twitch_irc::transport::tcp::TLS>,
                    twitch_irc::login::StaticLoginCredentials,
                >,
            >,
        >,
    ) {
        let receiver = self.nvim.session.start_event_loop_channel();

        for (event, values) in receiver {
            // TODO(Buser): Figure out how we exit this shit
            // if self.end {
            //     break;
            // }

            match Messages::from(event.clone()) {
                Messages::Test => {
                    self.nvim.command("echo \"testing\"").unwrap();
                }
                Messages::Init => {
                    let mut error = false;
                    let parsed_values: Vec<&str> = values
                        .iter()
                        .map(|v| {
                            let possible_value = v.as_str();

                            if possible_value.is_none() {
                                error = true;
                                return "error::default";
                            }

                            return possible_value.unwrap();
                        })
                        .collect();

                    if error {
                        self.nvim
                            .command(&format!("echo \"Error parsing values\"",))
                            .unwrap();
                    }
                    let mut args = parsed_values.iter();

                    let nickname = args.next().unwrap_or(&"monologue_mind").to_string();
                    let client_id = args
                        .next()
                        .unwrap_or(&"jzy5ssncfqreqxewn978xmgw03jy5w")
                        .to_string();
                    let oauth_port = args.next().unwrap_or(&"6969").to_string();

                    self.twitch.nickname = Option::from(nickname);
                    self.twitch.client_id = Option::from(client_id);
                    self.oauth_port = oauth_port;

                    self.nvim
                        .command(&format!("echo \"Successfully ran TwitchInit, run TwitchOAuth to create a connection\"",))
                        .unwrap();
                }
                Messages::OAuth => {
                    if self.twitch.client_id.is_none() {
                        self.nvim
                            .command(
                                "echo \"client_id not set. Run ':Init nickname client_id oauth_port'\"",
                            )
                            .unwrap();
                    }

                    if self.twitch.access_token.is_some() {
                        self.nvim
                            .command("echo \"access_token already set and valid\"")
                            .unwrap();
                    }

                    // This is blocking
                    let result = oauth::local_connect(
                        self.twitch.client_id.clone().unwrap(),
                        self.oauth_port.clone(),
                        &mut self.nvim,
                    );

                    match result {
                        Ok(access_token) => {
                            self.nvim
                                .command(&format!("echo \"access_token: {access_token}\""))
                                .unwrap();
                            self.twitch.access_token = Option::from(access_token);

                            let mut config = twitch_irc::ClientConfig::default();
                            config.login_credentials = twitch_irc::login::StaticLoginCredentials {
                                credentials: twitch_irc::login::CredentialsPair {
                                    login: "".to_string(),
                                    token: self.twitch.access_token.clone(),
                                },
                            };

                            let updated_twitch_client = twitch_irc::TwitchIRCClient::<
                                twitch_irc::SecureTCPTransport,
                                twitch_irc::login::StaticLoginCredentials,
                            >::new(config);

                            let mut incoming_messages = incoming_messages_arc.lock().await;
                            *incoming_messages = updated_twitch_client.0;

                            let mut client = client_arc.lock().await;
                            *client = updated_twitch_client.1;

                            self.nvim
                                .command(&format!("echo \"Connected to Twitch\""))
                                .unwrap();
                        }
                        Err(e) => {
                            self.nvim
                                .command(&format!("echo \"Error authing: {e}\""))
                                .unwrap();
                        }
                    }
                }
                Messages::Join => {
                    if self.twitch.nickname.is_none() || self.twitch.access_token.is_none() {
                        self.nvim
                            .command(&format!(
                                "echo \"Settigs valid: nickname: {}, access_token: {}\"",
                                self.twitch.nickname.is_some(),
                                self.twitch.access_token.is_some()
                            ))
                            .unwrap();
                    }

                    if self.client.is_none() {
                        self.nvim
                            .command("echo \"client has not been created, please run ':Oauth'\"")
                            .unwrap();
                    }

                    if !self.listening {
                        // TODO(Buser): Kick off listener????
                        let incoming_messages = Arc::clone(&incoming_messages_arc);
                        let join_handle = tokio::spawn(async move {
                            let mut messages = incoming_messages.lock().await;
                            while let Some(message) = messages.recv().await {
                                // println!("Received message: {:?}", message.clone());
                                // self.nvim.command(&format!("echo \"{message:?}\"")).unwrap();
                            }
                        });

                        join_handles.push(join_handle);
                    }

                    let mut args = values.iter();
                    let channel = args.next().unwrap().to_string();

                    // TODO(Buser): Need to figure out if a single socket
                    // handles all joins, if so we only create the listener once
                    let client = self.client.clone().unwrap();
                    let response = client.join(channel);

                    if let Err(e) = response {
                        self.nvim
                            .command(&format!("echo \"Error joining channel: {e:?}\""))
                            .unwrap();
                    }
                }
                // Messages::Exit => {
                //     // let mut args = values.iter();
                //     // let channel = args.next().unwrap().to_string();
                //
                //     // End listener
                //     for handle in self.join_handles {
                //         handle.abort();
                //     }
                // }
                Messages::Send => {
                    if self.twitch.nickname.is_none() || self.twitch.access_token.is_none() {
                        self.nvim
                            .command(&format!(
                                "echo \"Some settigs invalid: nickname: {}, access_token: {}\"",
                                self.twitch.nickname.is_none(),
                                self.twitch.access_token.is_none()
                            ))
                            .unwrap();
                    }

                    if self.client.is_none() {
                        self.nvim
                            .command("echo \"client has not been created, please run ':Oauth'\"")
                            .unwrap();
                    }

                    let mut args = values.iter();
                    let channel = args.next().unwrap().to_string();
                    let message = args.next().unwrap().to_string();

                    // TODO(Buser): Figure out how to get access to functions that require unwrap
                    // without a clone
                    let client = self.client.clone().unwrap();
                    let msg = client.say(channel.clone(), message.clone()).await;

                    if let Err(e) = msg {
                        self.nvim
                            .command(&format!("echo \"Error sending message: {e:?}\""))
                            .unwrap();
                    }
                }

                // Handle anything else
                Messages::Unknown(event) => {
                    let res = self.nvim.session.call(
                        "nvim_create_buf",
                        vec![Value::from(true), Value::from(false)],
                    );

                    if let Err(e) = res.clone() {
                        self.nvim
                            .command(&format!("echo \"Error creating buf: {}\"", e.to_string()))
                            .unwrap();
                    }

                    let value = res.unwrap();
                    self.nvim
                        .command(&format!("echo \"buf value: {}\"", value.to_string()))
                        .unwrap();

                    // self.nvim
                    //     .command(&format!("echo \"Setting up unknown\""))
                    //     .unwrap();
                    // let what = Buffer::new(Value::from(""));
                    // self.nvim
                    //     .command(&format!("echo \"building buffer\""))
                    //     .unwrap();
                    // // let res = what.set_name(&mut self.nvim, "sup");
                    // // self.nvim
                    // //     .command(&format!("echo \"setting name\""))
                    // //     .unwrap();
                    // // if let Err(e) = res {
                    // //     self.nvim
                    // //         .command(&format!("echo \"Error setting name: {}\"", e.to_string()))
                    // //         .unwrap();
                    // //     return;
                    // // }
                    // self.nvim
                    //     .command(&format!("echo \"before attaching\""))
                    //     .unwrap();
                    // let res = what.attach(&mut self.nvim, true, vec![]);
                    // self.nvim.command(&format!("echo \"attaching\"")).unwrap();
                    // if let Err(e) = res {
                    //     self.nvim
                    //         .command(&format!("echo \"Error attaching: {}\"", e.to_string()))
                    //         .unwrap();
                    // }

                    self.nvim
                        .command(&format!(
                            "echo \"Unknown command: {}. We support Init, Join, Send, OAuth\"",
                            event
                        ))
                        .unwrap();
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut event_handler = EventHandler::new();

    let default_config = twitch_irc::ClientConfig::default();
    let default_client = twitch_irc::TwitchIRCClient::<
        twitch_irc::SecureTCPTransport,
        twitch_irc::login::StaticLoginCredentials,
    >::new(default_config);

    let incoming_messages = Arc::new(Mutex::new(default_client.0));
    let client = Arc::new(Mutex::new(default_client.1));
    let buffers = Arc::new(Mutex::new(HashMap::<String, String>::new()));

    let mut join_handles: Vec<tokio::task::JoinHandle<()>> = vec![];
    event_handler
        .recv(&mut join_handles, incoming_messages, client)
        .await;

    // abort remaining handles
    for handle in join_handles {
        handle.abort();
    }
}
