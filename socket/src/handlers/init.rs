use neovim_lib::{Neovim, NeovimApi, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InitValues {
    pub nickname: Option<String>,
    pub client_id: Option<String>,
    pub oauth_port: Option<String>,
}

pub async fn init(
    nvim: &mut Neovim,
    values: &Vec<Value>,
    chat_logs_folder_path_arc: &Arc<RwLock<String>>,
) -> Option<InitValues> {
    nvim.command(&format!("echo \"enter\"",)).unwrap();
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

    if parsed_values.len() != 4 {
        nvim.command(&format!("echo \"Requires 4 arguments\"",))
            .unwrap();
        return None;
    }

    let mut args = parsed_values.iter();

    // TODO(Buser): check for errors on unwrap
    let nickname = args.next().unwrap().to_string();
    let client_id = args.next().unwrap().to_string();
    let oauth_port = args.next().unwrap().to_string();
    let chat_logs_folder_path = args.next().unwrap().to_string();

    let mut handle = chat_logs_folder_path_arc.write().await;
    *handle = chat_logs_folder_path;

    return Option::from(InitValues {
        nickname: Option::from(nickname),
        client_id: Option::from(client_id),
        oauth_port: Option::from(oauth_port),
    });
}
