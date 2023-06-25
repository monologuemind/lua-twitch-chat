---@type { splitString: fun(name: string, delimiter: string): table; stringToBinary: fun(str: string): string; getOperatingSystem: fun(): string; tablelength: fun(T: table): integer }
local helpers = require "./helpers.lua"

---@param twitch_init fun(opts: { nickname: string, client_id: string, oauth_port: string, chat_log_path: string })
function ConfigureCommands(twitch_init)
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
    vim.rpcnotify(Twitch_JobId, Twitch_Oauth)
  end, { nargs = "?" })

  ---@param opts { args: string }
  vim.api.nvim_create_user_command("TwitchJoin", function(opts)
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
