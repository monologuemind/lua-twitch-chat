local helpers = require("lua-twitch-chat.helpers")

---@param twitch_init fun(opts: { nickname: string, client_id: string, oauth_port: string, chat_log_path: string })
local function configureCommands(twitch_init)
  --- @param opts { args: string }
  vim.api.nvim_create_user_command("TwitchView", function(opts)
    local args = helpers.splitString(opts.args or "", " ")
    vim.rpcnotify(Twitch_JobId, Twitch_View, args[1])
  end, { nargs = "?" })

  vim.api.nvim_create_user_command("TwitchExit", function()
    vim.rpcnotify(Twitch_JobId, Twitch_Exit)
    Twitch_JobId = 0;
    -- vim.fn.jobstop(Target_Application)
  end, {})

  vim.api.nvim_create_user_command("TwitchTest", function()
    vim.rpcnotify(Twitch_JobId, Twitch_Test)
  end, {})

  vim.api.nvim_create_user_command("TwitchUnknown", function()
    -- vim.rpcnotify(Twitch_JobId, Twitch_Unknown)
  end, {})

  ---@param opts { args: string }
  vim.api.nvim_create_user_command("TwitchInit", function(opts)
    local args = helpers.splitString(opts.args or "", " ")

    if helpers.tablelength(args) ~= 4 then
      vim.notify(
        "TwitchInit requires only 4 arguments: nickname client_id port chat_log_path",
        vim.log.levels.ERROR)
      return
    end

    twitch_init({
      nickname = args[0],
      client_id = args[1],
      oauth_port = args[2],
      chat_log_path = args[3]
    })
  end, { nargs = "*" })

  vim.api.nvim_create_user_command("TwitchOAuth", function()
    if Twitch_JobId == 0 or Twitch_JobId == -1 then
      vim.cmd("TwitchSetup")
    end
    vim.rpcnotify(Twitch_JobId, Twitch_Oauth)
  end, { nargs = "?" })

  ---@param opts { args: string }
  vim.api.nvim_create_user_command("TwitchJoin", function(opts)
    if Twitch_Authed == false then vim.cmd("TwitchOAuth") end

    local args = helpers.splitString(opts.args or "", " ")

    if helpers.tablelength(args) == 0 then
      vim.notify("No arguments passed", vim.log.levels.ERROR)
      return
    end

    vim.rpcnotify(Twitch_JobId, Twitch_Join, args[1])
  end, { nargs = "?" })

  vim.api.nvim_create_user_command("TwitchBuf", function()
    local buf = vim.api.nvim_create_buf(true, false)
    vim.api.nvim_buf_set_name(buf, "MyBuffer")

    -- Set the initial data in the buffer
    local data = "Hello, world!"
    vim.api.nvim_buf_set_lines(buf, 0, -1, false, { data })

    -- Open the buffer in a new split window
    vim.cmd("split")
    vim.api.nvim_win_set_buf(0, buf)
  end, { nargs = "?" })
end

return { configureCommands = configureCommands }
