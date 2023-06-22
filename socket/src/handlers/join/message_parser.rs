use std::collections::HashMap;
use std::{fs::OpenOptions, io::Write};
use tokio::sync::RwLockReadGuard;
use twitch_irc::message::ServerMessage;

pub struct ChannelData {
    // May not always be viewing the chat logs
    pub buffer_id: Option<String>,
    // Will always be writing them to a file
    // Unless we leave in which case struct is destroyed
    pub file_name: String,
    pub highlight_name: String,
}

pub fn debug_write(data: String, debug: bool) {
    if !debug {
        return;
    }
    let file_path = "/home/michaelbuser/Documents/chat_logs/debug.chat".to_string();
    let debug_file_exists = std::path::Path::new(&file_path).exists();

    if debug_file_exists {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(file_path.clone())
            .unwrap();

        if let Err(e) = file.write_all(data.as_bytes()) {
            // TODO(Buser): Do something about this one
        }
    }

    let mut file = std::fs::File::create(file_path.clone()).unwrap();

    if let Err(e) = file.write_all(data.as_bytes()) {
        // TODO(Buser): Do something about this one
    }
}

pub fn handle_file(file_path: String, data: String) {
    let file_exists = std::path::Path::new(&file_path).exists();

    if file_exists {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(file_path.clone())
            .unwrap();

        if let Err(e) = file.write_all(data.as_bytes()) {
            // TODO(Buser): Do something about this one
        }
        return;
    }

    let mut file = std::fs::File::create(file_path.clone()).unwrap();

    if let Err(e) = file.write_all(data.as_bytes()) {
        // TODO(Buser): Do something about this one
    }
}

// TODO(Buser): Parse the message
// * get the associated buffer by channel name
// * check file length of ChannelData file_name
// * if max create new file and update ChannelData (might be an
//   issue with multiple locks going on, one read and one write)
// * append message to file
pub fn parse_message(
    message: ServerMessage,
    buffers: &RwLockReadGuard<HashMap<String, ChannelData>>,
    path: &RwLockReadGuard<String>,
    highlight_map: &mut HashMap<String, String>,
) {
    let debug = false;
    match message {
        ServerMessage::Privmsg(msg) => {
            let data = format!(
                "in {} -> {}: {}\n",
                msg.channel_login, msg.sender.name, msg.message_text
            );
            debug_write(data.clone(), debug);

            let channel_data = buffers.get(&msg.channel_login);

            if channel_data.is_none() {
                // TODO(Buser): Do something about this one
                debug_write("channel_data is none".to_string(), debug);
            }

            let chat_log_dir_path = path.clone();
            let chat_log_dir_path_exists =
                std::path::Path::new(&format!("{}/", chat_log_dir_path)).is_dir();

            if !chat_log_dir_path_exists {
                std::fs::create_dir(chat_log_dir_path.to_string()).unwrap();
            }

            let chat_log_file_path = channel_data.unwrap().file_name.clone();
            // let chat_log_file_exists = std::path::Path::new(&chat_log_file_path).exists();
            //
            // if chat_log_file_exists {
            //     let mut file = OpenOptions::new()
            //         .write(true)
            //         .append(true)
            //         .open(chat_log_file_path.clone())
            //         .unwrap();
            //
            //     if let Err(e) = file.write_all(data.as_bytes()) {
            //         // TODO(Buser): Do something about this one
            //         debug_write("error appending".to_string(), debug);
            //     }
            //     return;
            // }
            handle_file(chat_log_file_path, data);
            {
                let user_current_color = highlight_map.get(&msg.sender.id);
                if user_current_color.is_none() && msg.name_color.is_some() {
                    // ADD TO HASHMAP AND FILE
                    handle_file(
                        channel_data.unwrap().highlight_name.clone(),
                        format!(
                            "{},{}\n",
                            msg.sender.name,
                            msg.name_color.unwrap().to_string()
                        ),
                    );

                    highlight_map.insert(msg.sender.name, msg.name_color.unwrap().to_string());
                }
            }

            // let user_current_color = highlight_map.get(&msg.sender.id);
            // if user_current_color.is_some()
            // && msg.name_color.is_some()
            // && user_current_color.unwrap() != &msg.name_color.unwrap().to_string()
            // {
            //                // UPDATE HASHMAP AND FILE
            // }

            // let mut file = std::fs::File::create(chat_log_file_path).unwrap();
            //
            // if let Err(e) = file.write_all(data.as_bytes()) {
            //     // TODO(Buser): Do something about this one
            //     debug_write("error initial writing".to_string(), debug);
            // }
        }

        _ => {
            // let tags = message.source().clone().tags.0;
            // let prefix = message.source().clone().prefix.unwrap();
            // let params = message.source().clone().params;
            // let command = message.source().clone().command;
            //
            // let data = format!(
            //     "BUNK: {tags:?}@{prefix:?}@{params:?}@{command:?}\n"
            // );
            // // handle_file("/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket".to_string(), "/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/chat.log".to_string(), data.clone());
            // // let _ = std::fs::write("/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/recent.log", data);
            // let mut file = OpenOptions::new()
            //     .write(true)
            //     .append(true)
            //     .open("/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/chat.log")
            //     .unwrap();
            //
            // if let Err(e) = file.write_all(data.as_bytes()) {}
        }
    }
}
