use chrono::{Datelike, Local, Timelike};
use neovim_lib::{Neovim, NeovimApi, Session, Value};
use std::{collections::HashMap, sync::Arc};
use std::{fs::OpenOptions, io::Write};
use tokio::sync::{Mutex, RwLock};
use twitch_irc::message::ServerMessage;

mod buffer;
mod oauth;

fn format_date(date: chrono::DateTime<Local>) -> String {
    format!(
        "{:02}:{:02}_{}_{}_{}",
        date.hour(),
        date.minute(),
        date.day(),
        date.month(),
        date.year()
    )
}

fn handle_file(chat_log_file_path: String, chat_log_dir_path: String, data: String) {
    let chat_log_file_exists = std::path::Path::new(&chat_log_file_path).exists();

    if chat_log_file_exists {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(chat_log_file_path.clone())
            .unwrap();

        if let Err(e) = file.write_all(data.as_bytes()) {
            // eprintln!("Error writing to logfile:\n{e:?}");
        }
    }

    let chat_log_dir_path_exists = std::path::Path::new(&chat_log_dir_path).is_dir();

    if !chat_log_dir_path_exists {
        std::fs::create_dir(chat_log_dir_path).unwrap();
    }

    let mut file = std::fs::File::create(chat_log_file_path).unwrap();

    if let Err(e) = file.write_all(data.as_bytes()) {
        // eprintln!("Error writing to logfile:\n{e:?}");
    }
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

struct ChannelData {
    // May not always be viewing the chat logs
    buffer_id: Option<String>,
    // Will always be writing them to a file
    // Unless we leave in which case struct is destroyed
    file_name: String,
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
            RwLock<
                twitch_irc::TwitchIRCClient<
                    twitch_irc::transport::tcp::TCPTransport<twitch_irc::transport::tcp::TLS>,
                    twitch_irc::login::StaticLoginCredentials,
                >,
            >,
        >,
        chat_logs_folder_path_arc: Arc<RwLock<String>>,
        buffers: Arc<RwLock<HashMap<String, ChannelData>>>,
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
                    self.nvim.command(&format!("echo \"enter\"",)).unwrap();
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
                        return;
                    }

                    if parsed_values.len() != 4 {
                        self.nvim
                            .command(&format!("echo \"Requires 4 arguments\"",))
                            .unwrap();
                        return;
                    }

                    let mut args = parsed_values.iter();

                    self.nvim.command(&format!("echo \"args\"",)).unwrap();
                    // TODO(Buser): check for errors on unwrap
                    let nickname = args.next().unwrap().to_string();
                    let client_id = args.next().unwrap().to_string();
                    let oauth_port = args.next().unwrap().to_string();
                    let chat_logs_folder_path = args.next().unwrap().to_string();

                    let mut handle = chat_logs_folder_path_arc.write().await;
                    *handle = chat_logs_folder_path;

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
                        return;
                    }

                    if self.twitch.access_token.is_some() {
                        self.nvim
                            .command("echo \"access_token already set and valid\"")
                            .unwrap();
                        return;
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
                                    // login: "monologuemind".to_string(),
                                    login: self.twitch.nickname.clone().unwrap(),
                                    token: self.twitch.access_token.clone(),
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
                        return;
                    }

                    // TODO(Buser): Add leave command
                    // Certainly! To leave a channel using IRC WebSocket messages, you can send the appropriate IRC command. Here's an example of how you can do it:
                    //
                    // 1. Connect to the IRC server and join the channel you want to leave.
                    //    - Send: `PASS <oauth_token>` (if applicable)
                    //    - Send: `NICK <your_bot_username>`
                    //    - Send: `JOIN #<channel_name>`
                    //
                    // 2. Once you are connected and joined the channel, you can send the PART command to leave the channel.
                    //    - Send: `PART #<channel_name>`
                    //
                    // By sending the PART command, your bot will leave the specified channel.
                    //
                    // Make sure you replace `<oauth_token>` with your actual OAuth token (if required), `<your_bot_username>` with your bot's username, and `<channel_name>` with the name of the channel you want to leave.
                    //
                    // Remember to implement the appropriate WebSocket connection and message handling logic in your bot's code. The exact implementation details may vary depending on the programming language and library you are using for the IRC WebSocket communication.
                    if self.twitch.nickname.is_none() || self.twitch.access_token.is_none() {
                        self.nvim
                            .command(&format!(
                                "echo \"Settigs valid: nickname: {}, access_token: {}\"",
                                self.twitch.nickname.is_some(),
                                self.twitch.access_token.is_some(),
                            ))
                            .unwrap();
                        return;
                    }

                    // if self.client.is_none() {
                    //     self.nvim
                    //         .command("echo \"client has not been created, please run ':Oauth'\"")
                    //         .unwrap();
                    //     return;
                    // }

                    let mut args = parsed_values.iter();
                    let channel = args.next().unwrap().to_string();

                    let path = chat_logs_folder_path_arc.read().await;
                    let date = format_date(Local::now());
                    let file_name = format!("{}/{channel}-{date}.chat.md", path.to_string());
                    {
                        let mut buffer_guard = buffers.write().await;
                        buffer_guard.insert(
                            channel.clone(),
                            ChannelData {
                                buffer_id: None,
                                file_name,
                            },
                        );
                        drop(buffer_guard);
                    }

                    let _ = std::fs::write("/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/chat.log", "");
                    if !self.listening {
                        let incoming_messages = Arc::clone(&incoming_messages_arc);
                        let buffer_arc_clone = Arc::clone(&buffers);

                        let join_handle = tokio::spawn(async move {
                            let mut messages = incoming_messages.lock().await;
                            let buffers = buffer_arc_clone.read().await;
                            while let Some(message) = messages.recv().await {
                                // message.source();

                                match message {
                                    ServerMessage::Privmsg(msg) => {
                                        let data = format!(
                                            "MSG: {}@{}: {}\n",
                                            msg.channel_login, msg.sender.name, msg.message_text
                                        );
                                        // handle_file("/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket".to_string(), "/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/chat.log".to_string(), data.clone());
                                        // let _ = std::fs::write("/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/recent.log", data);
                                        let mut file = OpenOptions::new()
                                            .write(true)
                                            .append(true)
                                            .open("/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/chat.log")
                                            .unwrap();

                                        if let Err(e) = file.write_all(data.as_bytes()) {}
                                    }
                                    _ => {
                                        let tags = message.source().clone().tags.0;
                                        let prefix = message.source().clone().prefix.unwrap();
                                        let params = message.source().clone().params;
                                        let command = message.source().clone().command;

                                        let data = format!(
                                            "BUNK: {tags:?}@{prefix:?}@{params:?}@{command:?}\n"
                                        );
                                        // handle_file("/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket".to_string(), "/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/chat.log".to_string(), data.clone());
                                        // let _ = std::fs::write("/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/recent.log", data);
                                        let mut file = OpenOptions::new()
                                            .write(true)
                                            .append(true)
                                            .open("/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/chat.log")
                                            .unwrap();

                                        if let Err(e) = file.write_all(data.as_bytes()) {}
                                    }
                                }

                                // TODO(Buser): Parse the message
                                // * get the associated buffer by channel name
                                // * check file length of ChannelData file_name
                                // * if max create new file and update ChannelData (might be an
                                //   issue with multiple locks going on, one read and one write)
                                // * append message to file
                            }
                        });

                        join_handles.push(join_handle);
                    }

                    // TODO(Buser): Need to figure out if a single socket
                    // handles all joins, if so we only create the listener once
                    self.nvim
                        .command(format!("echo \"channel: {channel}\"").as_str())
                        .unwrap();
                    let client = client_arc.write().await;
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
                    // let buf = self.nvim.session.call(
                    //     "nvim_create_buf",
                    //     vec![Value::from(true), Value::from(false)],
                    // );
                    //
                    // if let Err(e) = buf.clone() {
                    //     self.nvim
                    //         .command(&format!("echo \"Error creating buf: {}\"", e.to_string()))
                    //         .unwrap();
                    //     return;
                    // }
                    //
                    // let value = buf.unwrap();
                    //
                    // let res = self.nvim.session.call(
                    //     "nvim_buf_set_name",
                    //     vec![
                    //         Value::from(value.clone()),
                    //         Value::from("some_name".to_string()),
                    //     ],
                    // );
                    //
                    // if let Err(e) = res {
                    //     self.nvim
                    //         .command(&format!("echo \"Error setting name: {}\"", e.to_string()))
                    //         .unwrap();
                    //     return;
                    // }
                    //
                    // let lines = vec![Value::from("some data"), Value::from("row two")];
                    // let res = self.nvim.session.call(
                    //     "nvim_buf_set_lines",
                    //     vec![
                    //         value.clone(),
                    //         Value::from(0),
                    //         Value::from(-1),
                    //         Value::from(false),
                    //         Value::from(lines),
                    //     ],
                    // );
                    //
                    // if let Err(e) = res {
                    //     self.nvim
                    //         .command(&format!("echo \"Error setting lines: {}\"", e.to_string()))
                    //         .unwrap();
                    //     return;
                    // }
                    //
                    // self.nvim.command("vsplit").unwrap();
                    //
                    // let res = self
                    //     .nvim
                    //     .session
                    //     .call("nvim_win_set_buf", vec![Value::from(0), value.clone()]);
                    //
                    // if let Err(e) = res {
                    //     self.nvim
                    //         .command(&format!("echo \"Error setting buf: {}\"", e.to_string()))
                    //         .unwrap();
                    //     return;
                    // }
                    //
                    // let res = self
                    //     .nvim
                    //     .session
                    //     .call("nvim_buf_line_count", vec![value.clone()]);
                    //
                    // if let Err(e) = res {
                    //     self.nvim
                    //         .command(&format!("echo \"Error setting buf: {}\"", e.to_string()))
                    //         .unwrap();
                    //     return;
                    // }
                    //
                    // let buf_line_count = res.unwrap();
                    //
                    // let more_lines = vec![Value::from("more data"), Value::from("even more data")];
                    // let res = self.nvim.session.call(
                    //     "nvim_buf_set_lines",
                    //     vec![
                    //         value.clone(),
                    //         buf_line_count.clone(),
                    //         buf_line_count.clone(),
                    //         Value::from(true),
                    //         Value::from(more_lines),
                    //     ],
                    // );
                    //
                    // if let Err(e) = res {
                    //     self.nvim
                    //         .command(&format!("echo \"Error setting buf: {}\"", e.to_string()))
                    //         .unwrap();
                    //     return;
                    // }

                    // TODO(Buser): Implement this lua code to append lines
                    // vim.api.nvim_buf_set_lines()
                    // vim.api.nvim_buf_line_count()
                    // -- Append lines to the buffer
                    // local function appendLines(lines)
                    //   if _G.myBuffer.id ~= nil then
                    //     local buf = _G.myBuffer.id
                    //
                    //     -- Get the current lines in the buffer
                    //     local currentLines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
                    //
                    //     -- Append the new lines
                    //     local newLines = {}
                    //     for _, line in ipairs(lines) do
                    //       table.insert(newLines, line)
                    //     end
                    //
                    //     -- Concatenate the current and new lines
                    //     local updatedLines = vim.list_extend(currentLines, newLines)
                    //
                    //     -- Update the buffer with the updated lines
                    //     vim.api.nvim_buf_set_lines(buf, 0, -1, false, updatedLines)
                    //   else
                    //     print("Buffer not found. Please create the buffer first.")
                    //   end
                    // end

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
    let client = Arc::new(RwLock::new(default_client.1));
    let chat_logs_folder_path = Arc::new(RwLock::new("".to_string()));
    let buffers = Arc::new(RwLock::new(HashMap::<String, ChannelData>::new()));

    let mut join_handles: Vec<tokio::task::JoinHandle<()>> = vec![];
    event_handler
        .recv(
            &mut join_handles,
            incoming_messages,
            client,
            chat_logs_folder_path,
            buffers,
        )
        .await;

    // abort remaining handles
    for handle in join_handles {
        handle.abort();
    }
}
