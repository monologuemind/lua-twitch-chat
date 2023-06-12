use neovim_lib::{Neovim, NeovimApi, Session, Value};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

mod buffer;
mod handlers;

enum Messages {
    Test,
    Init,
    Join,
    View,
    Send,
    OAuth,
    Exit,
    Unknown(String),
}

impl From<String> for Messages {
    fn from(event: String) -> Self {
        match &event[..] {
            "test" => Messages::Test,
            "init" => Messages::Init,
            "join" => Messages::Join,
            "exit" => Messages::Exit,
            "send" => Messages::Send,
            "view" => Messages::View,
            "oauth" => Messages::OAuth,
            _ => Messages::Unknown(event),
        }
    }
}

struct Twitch {
    nickname: Option<String>,
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
    end: bool,
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
            end: false,
        }
    }

    async fn recv<'a>(
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
        buffers: Arc<RwLock<HashMap<String, handlers::join::message_parser::ChannelData>>>,
    ) {
        let receiver = self.nvim.session.start_event_loop_channel();

        for (event, values) in receiver {
            // TODO(Buser): Figure out how we exit this shit
            if self.end {
                break;
            }

            match Messages::from(event.clone()) {
                Messages::Test => {
                    self.nvim.command("echo \"testing\"").unwrap();
                }
                Messages::Init => {
                    let possible_init_values =
                        handlers::init::init(&mut self.nvim, &values, &chat_logs_folder_path_arc)
                            .await;

                    if possible_init_values.is_some() {
                        let init_values = possible_init_values.unwrap();

                        self.twitch.client_id = init_values.client_id.clone();
                        self.twitch.nickname = init_values.client_id.clone();
                        self.oauth_port = init_values.oauth_port.unwrap();
                    }
                }
                Messages::OAuth => {
                    if self.twitch.client_id.is_none() {
                        self.nvim.command(
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
                    let access_token = handlers::oauth::oauth(
                        &self.twitch.client_id,
                        &self.twitch.nickname,
                        &self.oauth_port,
                        &mut self.nvim,
                        &incoming_messages_arc,
                        &client_arc,
                    )
                    .await;

                    if access_token.is_some() {
                        self.twitch.access_token = access_token;
                    }
                }
                Messages::Join => {
                    let listening = handlers::join::join(
                        self.listening,
                        join_handles,
                        &chat_logs_folder_path_arc,
                        &buffers,
                        &values,
                        &self.twitch.access_token,
                        &self.twitch.nickname,
                        &mut self.nvim,
                        &incoming_messages_arc,
                        &client_arc,
                    )
                    .await;
                    if !self.listening && listening.is_some() {
                        self.listening = true;
                    }
                }

                Messages::View => {
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

                    let mut args = parsed_values.iter();
                    let channel = args.next().unwrap().to_string();

                    let buf = buffers.read().await;
                    let channel_data = buf.get(&channel);

                    if channel_data.is_none() {
                        self.nvim
                            .command(&format!(
                                "echo \"Error finding channel, try :TwitchJoin channel_name\"",
                            ))
                            .unwrap();
                        return;
                    }

                    let file_name = channel_data.unwrap().file_name.clone();
                    self.nvim.command(&format!("e {file_name}")).unwrap();
                }

                Messages::Exit => {
                    for handle in join_handles.iter() {
                        handle.abort();
                    }

                    self.end = true;
                }

                Messages::Send => {
                    handlers::send::say(
                        &self.twitch.nickname,
                        &self.twitch.access_token,
                        &mut self.nvim,
                        &self.client,
                        &values,
                    )
                    .await;
                }

                // Handle anything else
                Messages::Unknown(event) => {
                    // let buf = nvim.session.call(
                    //     "nvim_create_buf",
                    //     vec![Value::from(true), Value::from(false)],
                    // );
                    //
                    // if let Err(e) = buf.clone() {
                    //     nvim
                    //         .command(&format!("echo \"Error creating buf: {}\"", e.to_string()))
                    //         .unwrap();
                    //     return;
                    // }
                    //
                    // let value = buf.unwrap();
                    //
                    // let res = nvim.session.call(
                    //     "nvim_buf_set_name",
                    //     vec![
                    //         Value::from(value.clone()),
                    //         Value::from("some_name".to_string()),
                    //     ],
                    // );
                    //
                    // if let Err(e) = res {
                    //     nvim
                    //         .command(&format!("echo \"Error setting name: {}\"", e.to_string()))
                    //         .unwrap();
                    //     return;
                    // }
                    //
                    // let lines = vec![Value::from("some data"), Value::from("row two")];
                    // let res = nvim.session.call(
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
                    //     nvim
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
                    //     nvim
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
                    //     nvim
                    //         .command(&format!("echo \"Error setting buf: {}\"", e.to_string()))
                    //         .unwrap();
                    //     return;
                    // }
                    //
                    // let buf_line_count = res.unwrap();
                    //
                    // let more_lines = vec![Value::from("more data"), Value::from("even more data")];
                    // let res = nvim.session.call(
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
                    //     nvim
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
    let buffers = Arc::new(RwLock::new(HashMap::<
        String,
        handlers::join::message_parser::ChannelData,
    >::new()));

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
