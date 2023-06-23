use rand::Rng;
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

fn get_random_color() -> String {
    let mut rng = rand::thread_rng();
    let hex_chars: Vec<char> = "0123456789abcdef".chars().collect();

    let mut result = String::new();
    for _ in 0..6 {
        let random_index = rng.gen_range(0..16);
        result.push(hex_chars[random_index]);
    }

    result
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

pub fn parse_message(
    message: ServerMessage,
    buffers: &RwLockReadGuard<HashMap<String, ChannelData>>,
    path: &RwLockReadGuard<String>,
    highlight_map: &mut HashMap<String, String>,
) {
    let debug = false;
    match message {
        ServerMessage::Privmsg(msg) => {
            let data = format!("{}: {}\n", msg.sender.name, msg.message_text);
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
            handle_file(chat_log_file_path, data);

            {
                let user_current_color = highlight_map.get(&msg.sender.name);
                if user_current_color.is_none() {
                    // ADD TO HASHMAP AND FILE
                    let new_color = if msg.name_color.is_some() {
                        msg.name_color.unwrap().to_string()
                    } else {
                        format!("#{}", get_random_color()).to_string()
                    };

                    handle_file(
                        channel_data.unwrap().highlight_name.clone(),
                        format!("{},{}\n", msg.sender.name, new_color),
                    );

                    highlight_map.insert(msg.sender.name.clone(), new_color);
                }
            }

            {
                let user_current_color = highlight_map.get(&msg.sender.name);
                if user_current_color.is_some()
                    && msg.name_color.is_some()
                    && user_current_color.unwrap() != &msg.name_color.unwrap().to_string()
                {
                    let data =
                        std::fs::read_to_string(channel_data.unwrap().highlight_name.clone())
                            .unwrap();
                    let from = format!("{},{}\n", msg.sender.name, user_current_color.unwrap());
                    let to = format!(
                        "{},{}\n",
                        msg.sender.name,
                        &msg.name_color.unwrap().to_string()
                    );
                    let new = data.replace(from.as_str(), to.as_str());
                    // UPDATE HASHMAP AND FILE
                    let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(channel_data.unwrap().highlight_name.clone())
                        .unwrap();
                    file.write(new.as_bytes()).unwrap();

                    *highlight_map.get_mut(&msg.sender.name.clone()).unwrap() =
                        msg.name_color.unwrap().to_string();
                }
            }
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
