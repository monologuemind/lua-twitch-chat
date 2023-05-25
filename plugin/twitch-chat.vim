" Load only once
if exists('g:loaded_twitch_chat') | finish | endif

command! TwitchChatLoad lua require('twitch-chat').compile()
" command! NightfoxInteractive lua require('nightfox.interactive').attach()

let g:loaded_twitch_chat = 1
