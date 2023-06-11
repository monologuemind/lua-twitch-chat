use neovim_lib::{Neovim, NeovimApi, Value};

pub async fn say(
    nickname: &Option<String>,
    access_token: &Option<String>,
    nvim: &mut Neovim,
    client: &Option<
        twitch_irc::TwitchIRCClient<
            twitch_irc::SecureTCPTransport,
            twitch_irc::login::StaticLoginCredentials,
        >,
    >,
    values: &Vec<Value>,
) {
    if nickname.is_none() || access_token.is_none() {
        nvim.command(&format!(
            "echo \"Some settigs invalid: nickname: {}, access_token: {}\"",
            nickname.is_none(),
            access_token.is_none()
        ))
        .unwrap();
    }

    if client.is_none() {
        nvim.command("echo \"client has not been created, please run ':Oauth'\"")
            .unwrap();
    }

    let mut args = values.iter();
    let channel = args.next().unwrap().to_string();
    let message = args.next().unwrap().to_string();

    // TODO(Buser): Figure out how to get access to functions that require unwrap
    // without a clone
    let client = client.clone().unwrap();
    let msg = client.say(channel.clone(), message.clone()).await;

    if let Err(e) = msg {
        nvim.command(&format!("echo \"Error sending message: {e:?}\""))
            .unwrap();
    }
}
