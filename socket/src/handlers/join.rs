use chrono::{Datelike, Local, Timelike};
use neovim_lib::{Neovim, NeovimApi};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod message_parser;

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

pub async fn join(
    listening: bool,
    join_handles: &mut Vec<tokio::task::JoinHandle<()>>,
    chat_logs_folder_path_arc: &Arc<RwLock<String>>,
    buffers: &Arc<RwLock<std::collections::HashMap<String, message_parser::ChannelData>>>,
    values: &Vec<neovim_lib::Value>,
    access_token: &Option<String>,
    nickname: &Option<String>,
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
) -> Option<bool> {
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
        nvim.command(&format!("echo \"Error parsing values\"",))
            .unwrap();
        return None;
    }

    if nickname.is_none() || access_token.is_none() {
        nvim.command(&format!(
            "echo \"Settigs valid: nickname: {}, access_token: {}\"",
            nickname.is_some(),
            access_token.is_some(),
        ))
        .unwrap();
        return None;
    }

    let mut args = parsed_values.iter();
    let channel = args.next().unwrap().to_string();

    let buffer_guard = buffers.read().await;

    if buffer_guard.contains_key(&channel.clone()) {
        return None;
    }

    drop(buffer_guard);

    let path = chat_logs_folder_path_arc.read().await;
    let date = format_date(Local::now());
    let file_name = format!("{}/{channel}-{date}.chat", path.clone().to_string());
    message_parser::handle_file(file_name.clone().to_string(), "".to_string());

    {
        let mut buffer_guard = buffers.write().await;
        buffer_guard.insert(
            channel.clone(),
            message_parser::ChannelData {
                buffer_id: None,
                file_name: file_name.clone(),
            },
        );
        drop(buffer_guard);
    }

    if !listening {
        let incoming_messages = Arc::clone(&incoming_messages_arc);
        let buffer_arc_clone = Arc::clone(&buffers);
        let chat_logs_folder_path_clone = Arc::clone(&chat_logs_folder_path_arc);

        let join_handle = tokio::spawn(async move {
            let mut messages = incoming_messages.lock().await;
            let buffers = buffer_arc_clone.read().await;
            let path = chat_logs_folder_path_clone.read().await;

            while let Some(message) = messages.recv().await {
                message_parser::parse_message(message, &buffers, &path);
            }
        });

        join_handles.push(join_handle);
    }

    // TODO(Buser): Need to figure out if a single socket
    // handles all joins, if so we only create the listener once
    let client = client_arc.write().await;
    let response = client.join(channel.clone());
    let _ = nvim.command(&format!("lua vim.cmd.edit(\"{file_name}\")"));
    // nvim.command(
    //     format!("echo \"channel joined: {channel}, path: {path}, file_name: {file_name}\"")
    //         .as_str(),
    // )
    // .unwrap();

    if let Err(e) = response {
        nvim.command(&format!("echo \"Error joining channel: {e:?}\""))
            .unwrap();
        return None;
    }
    // | WatchFile
    // let _ = std::fs::File::create(file_name.clone()).unwrap();
    // nvim.command(format!("e \"{file_name}\"").as_str()).unwrap();

    return Option::from(true);
}
