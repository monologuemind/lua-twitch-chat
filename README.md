# twitch-chat.nvim

A twitch chat client for neovim

# Note

- This is the first time I have done anything with neovim plugins, rust, or lua (other than neovim config)
- I have only tested this on my local machine so your mileage will almost certainly vary
- A good portion of this code was taken from documentation of the crates depended on, as well as a good portion generated by ChatGPT (Most of it get's edited but it's possible I missed something bad that it has give me)
- Some of the in progress features have been rather difficult for me to even approach so any information or help is appreciated

# Features

- [x] Twitch OAuth login
- [x] Twitch user name highlighting
- [x] Twitch chat log (just a file with auto watch and auto scroll, so integrates with other plugins well)
- [ ] :construction: Twitch chat log auto scroll
  - Works when chat log is current buffer but otherwise does not
- [ ] :construction: Twitch emote support
  - I have tried a few options without much progress:
    - Attempt at converting images into icons for an icon font (large SVG paths and either lack of or no color/multi-color support)
    - Looked at image rendering in terminals (nothing consistent between terminal options)
- [ ] Figure out how to handle the rust portion of this plugin (precompile, or add instructions for user to compile)
- [ ] Add support for joining multiple channels at a time
- [ ] Add support for sending messages in channels (including commands)
- [ ] Add support for a combined view of the multiple channels
- [ ] Look into neovim feature hacks ("hover" over user and reply/ban/timeout)
- [ ] Additional Twitch API support (there is a lot that this could be broken down into)

# Setup

You will need to setup your own [twitch application](https://dev.twitch.tv/docs/api/get-started/)

- It should be in the chat bot category
- The OAuth redirect URL should be something like http://localhost:6969 (note that your port is based on what you use in the setup function)
- You will need to copy your Client Id that they generate for you for your setup function

You will need to compile the rust code. There is a feature line item to figure out what we should do to make this process easier.

You will then need to add the plugin to your lua user files. There a few different plugin managers however the setup function will be the same in all of them

```lua
    use "monologuemind/twitch-chat.nvim"

    require("twitch-chat").setup({
        nickname = "monologue_mind",
        client_id = "YOUR_CLIENT_ID_HERE",
        oauth_port = "6969",
        -- This will be where we store chat logs and user highlights per twitch stream
        chat_log_path = "/home/user/Documents/chat_logs",
        -- I suggest keeping this false so that you don't always have the rust process running
        auto_start = false,
        -- This is where the compiled version of the rust rpc backend lives.
        target_application = '/home/user/Documents/twitch-chat.nvim/socket/target/debug/socket'
    })
```

# Command list (partial)

| Command     | Description                                                                                      |
| ----------- | ------------------------------------------------------------------------------------------------ |
| TwitchOAuth | Starts a server to handle OAuth process, and opens a webpage to the redirect url                 |
| TwitchJoin  | Joins a twitch chat, ":TwitchJoin nl_kripp". Will also initiate TwitchOAuth if it hasn't started |
| TwitchView  | Attempts to view the current chat log for a channel, ":TwitchView nl_kripp"                      |
