-- function getOperatingSystem()
--   local osName = string.lower((ffi and ffi.os) or (os and os.getenv("OS")) or (io and io.popen("uname -s"):read("*l")))
--
--   if osName:match("linux") then
--     return "Linux"
--   elseif osName:match("darwin") then
--     return "Mac"
--   elseif osName:match("windows") then
--     return "Windows"
--   else
--     return "Unknown"
--   end
-- end
-- print(getOperatingSystem())
-- WATCH FILE --
-- local w = vim.loop.new_fs_event()
-- local function on_change(err, fname, status)
--   -- Do work...
--   vim.api.nvim_command('checktime')
--   CONDITIONS --
--   * check if the cursor is at bottom (nvim + ./some/file.txt)
--   * essentially if an update comes in then we ensure the cursor is at the bottom
--     unless the cursor was moved manually somewhere that isn't the bottom
--   * { buf: bufId, is_active: bool, cursor_at_bottom: bool }
--   * if not active buffer than we simply check if the cursor was last at the bottom
--   CONDITIONS --
--   -- Debounce: stop/start.
--   w:stop()
--   watch_file(fname)
-- end
-- function watch_file(fname)
--   local fullpath = vim.api.nvim_call_function(
--     'fnamemodify', { fname, ':p' })
--   w:start(fullpath, {}, vim.schedule_wrap(function(...)
--     on_change(...)
--   end))
-- end
--
-- vim.api.nvim_command(
--   "command! -nargs=1 Watch call luaeval('watch_file(_A)', expand('<args>'))")
-- AUTOSCROLL --
-- local function auto_scroll()
--     local bufnr = vim.api.nvim_get_current_buf()
--     local winnr = vim.api.nvim_get_current_win()
--
--     local prev_line_count = vim.api.nvim_buf_line_count(bufnr)
--
--     vim.api.nvim_buf_attach(bufnr, false, {
--         on_lines = function()
--             local current_line_count = vim.api.nvim_buf_line_count(bufnr)
--
--             if current_line_count > prev_line_count then
--                 local win_height = vim.api.nvim_win_get_height(winnr)
--                 local cursor_pos = vim.api.nvim_win_get_cursor(winnr)
--
--                 if cursor_pos[1] == win_height then
--                     vim.api.nvim_win_set_option(winnr, 'scroll', win_height - 1)
--                 end
--             end
--
--             prev_line_count = current_line_count
--         end
--     })
-- end
--
-- auto_scroll()
local function splitString(str, delimiter)
  local result = {}

  for match in string.gmatch(str, "[^" .. delimiter .. "]+") do
    table.insert(result, match)
  end

  return result
end

local function tablelength(T)
  local count = 0
  for _ in pairs(T) do count = count + 1 end
  return count
end

-- Initialize the channel
if not Twitch_JobId then Twitch_JobId = 0 end

-- Constants for RPC messages
Twitch_Oauth = 'oauth'
Twitch_Init = 'init'
Twitch_Test = 'test'
Twitch_Unknown = 'unknown'
Twitch_Join = 'join'

-- The path to the binary that was created out of 'cargo build' or 'cargo build --release'. This will generally be 'target/release/name'
Target_Application =
'/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/target/debug/socket'

MyTable = {}
-- MyTable.settings = {
--   file = "/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/target/debug/socket",
--   -- file = nil,
--   thread = nil
-- }

--[[--
@param opts @table
  @field nickname string
  @field client_id string
  @field oauth_port string
  @field chat_log_path string
--]]
local function twitch_init(opts)
  if Twitch_JobId == 0 or not opts.nickname or not opts.client_id or
      not opts.oauth_port or not opts.chat_log_path then
    vim.notify(string.format(
      "TwitchInit requires 4 arguments (nickname = %s client_id = %s oauth_port %s chat_log_path = %s)",
      opts.nickname, opts.client_id, opts.oauth_port,
      opts.chat_log_path), vim.log.levels.ERROR)
    return
  end

  vim.rpcnotify(Twitch_JobId, Twitch_Init, opts.nickname, opts.client_id,
    opts.oauth_port, opts.chat_log_path)
end

function ConfigureCommands()
  vim.api.nvim_create_user_command("TwitchTest", function()
    vim.rpcnotify(Twitch_JobId, Twitch_Test)
  end, {})
  vim.api.nvim_create_user_command("TwitchUnknown", function()
    vim.rpcnotify(Twitch_JobId, Twitch_Unknown)
  end, {})
  vim.api.nvim_create_user_command("TwitchInit", function(opts)
    local args = splitString(opts.args or "", " ")

    if tablelength(args) ~= 4 then
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
    -- vim.rpcnotify(Twitch_JobId, Twitch_Init, args[0], args[1], args[2], args[3])
  end, { nargs = "*" })

  vim.api.nvim_create_user_command("TwitchOAuth", function()
    vim.rpcnotify(Twitch_JobId, Twitch_Oauth)
  end, { nargs = "?" })

  vim.api.nvim_create_user_command("TwitchJoin", function(opts)
    local args = splitString(opts.args or "", " ")

    if tablelength(args) == 0 then
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

-- Initialize RPC
function InitTwitchRpc()
  if Twitch_JobId == 0 then
    Twitch_JobId = vim.fn.jobstart(Target_Application, { rpc = true })
  end
end

-- Entry point. Initialize RPC. If it succeeds, then attach commands to the `rpcnotify` invocations.
function Connect()
  InitTwitchRpc()

  if Twitch_JobId == 0 then
    print("Twitch: cannot start rpc process")
  elseif Twitch_JobId == -1 then
    print("Twitch: rpc process is not executable")
  else
    ConfigureCommands()
  end
end

--[[--
@param opts @table
@field nickname first string
@field client_id second string
@field oauth_port third string
@field chat_log_path fourth string
--]]
MyTable.setup = function(opts)
  -- Setting up the exit of the editor to also stop the socket
  local twitch_group = vim.api.nvim_create_augroup("TwitchSocket",
    { clear = true })
  vim.api.nvim_create_autocmd("ExitPre", {
    group = twitch_group,
    callback = function()
      if Twitch_JobId then vim.fn.jobstop(Twitch_JobId) end
    end
  })

  Connect()

  if Twitch_JobId and opts.nickname and opts.client_id and opts.oauth_port and
      opts.chat_log_path then
    twitch_init(opts)
    -- vim.rpcnotify(Twitch_JobId, Twitch_Init, opts.nickname, opts.client_id, opts.oauth_port, opts.chat_log_path)
  end
end

return MyTable
