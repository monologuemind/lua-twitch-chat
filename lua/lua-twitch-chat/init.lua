-- Initialize the channel
if not Twitch_JobId then Twitch_JobId = 0 end

-- Constants for RPC messages
Twitch_Oauth = 'oauth'
Twitch_Init = 'init'
Twitch_Test = 'test'
Twitch_Exit = 'exit'
Twitch_View = 'view'
Twitch_Unknown = 'unknown'
Twitch_Join = 'join'

-- The path to the binary that was created out of 'cargo build' or 'cargo build --release'. This will generally be 'target/release/name'
Target_Application =
'/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/target/debug/socket'

-- Just a table that we will return with some stuff on it
MyTable = {}

function GetOperatingSystem()
  local osName = string.lower((ffi and ffi.os) or (os and os.getenv("OS")) or
    (io and io.popen("uname -s"):read("*l")))

  if osName:match("linux") then
    return "Linux"
  elseif osName:match("darwin") then
    return "Mac"
  elseif osName:match("windows") then
    return "Windows"
  else
    return "Unknown"
  end
end

---@param str string
---@param delimiter string
local function splitString(str, delimiter)
  local result = {}

  for match in string.gmatch(str, "[^" .. delimiter .. "]+") do
    table.insert(result, match)
  end

  return result
end

---@param str string
local function stringToBinary(str)
  local binaryStr = ""

  -- Iterate over each character in the string
  for i = 1, #str do
    local char = str:sub(i, i)     -- Get the current character

    -- Convert the character to its ASCII value
    local asciiValue = string.byte(char)

    -- Convert the ASCII value to binary representation
    local binaryValue = string.format("%08d", tonumber(asciiValue, 10))

    -- Append the binary representation to the result
    binaryStr = binaryStr .. binaryValue
  end

  return binaryStr
end

--- @param data string
local load_highlights = function(data)
  ---@type { [string]: nil | string[] }
  local hex_groups = {}
  local user_colors = splitString(data, "\n")
  for i, v in pairs(user_colors) do
    local key_value_pair = splitString(v, ",")
    local group_exists = hex_groups[key_value_pair[2]]
    if group_exists == nil then hex_groups[key_value_pair[2]] = {} end
    table.insert(hex_groups[key_value_pair[2]], key_value_pair[1])
  end
  for hex_code, user_names in pairs(hex_groups) do
    local binary_code = stringToBinary(hex_code)
    vim.api.nvim_set_hl(0, binary_code, { fg = hex_code })
    for j, user_name in pairs(user_names) do
      local cm1 = "syntax keyword " .. user_name .. " " .. user_name
      vim.cmd(cm1)
      local cm2 = "highlight link " .. user_name .. " " .. binary_code
      vim.cmd(cm2)
    end
  end
end

-- WATCH FILE --
-- TODO: Allow for multiple chats
local w
local h

---@param fname string
---@param hname string
local function on_change(fname, hname)
  -- WAIT: Don't change this. It actually works if you pass a string
  local bufnr = vim.fn.bufnr(fname)
  local current_bufnr = vim.api.nvim_get_current_buf()
  vim.api.nvim_command('checktime')
  if bufnr == current_bufnr then
    vim.fn.cursor({ vim.fn.line('$'), 0 })
    local file = io.open(hname, "r")
    if (file) then
      local data = file:read("*a")
      if (data ~= nil and string.len(data) ~= h) then
        h = string.len(data)
        load_highlights(data)
      end
    end
  else
    -- THIS IS SUPER CLOSE: BLOCKING  AND ONLY WORKED UNTIL IT HAD TEXT BEYOND ITSELF
    vim.schedule(function()
      vim.cmd("silent execute 'buffer' " .. bufnr ..
        " | execute \"normal G\"")
      local file = io.open(hname, "r")
      if (file) then
        local data = file:read("*a")
        if (data ~= nil and string.len(data) ~= h) then
          h = string.len(data)
          load_highlights(data)
        end
      end
      vim.cmd("execute 'buffer' " .. current_bufnr)
    end)

    -- print(bufnr, current_bufnr)
    -- vim.cmd("execute 'silent! " .. bufnr .. " | normal! G'")
    -- vim.cmd("keepjumps silent! execute 'buffer' " .. bufnr ..
    --   " | silent! normal! G | execute 'normal! zz' | execute 'buffer' bufnr('#')");
  end

  w:stop()
  Watch_File(fname, hname)
end

---@param fname string
---@param hname string
function Watch_File(fname, hname)
  local fullpath = vim.api.nvim_call_function('fnamemodify', { fname, ':p' })
  w = vim.loop.new_fs_event()
  if w == nil then return end
  w:start(fullpath, {},
    vim.schedule_wrap(function() on_change(fname, hname) end))
end

vim.api.nvim_create_user_command("WatchFile", function(opts)
  local args = splitString(opts.args or "", " ")

  Watch_File(args[1], args[2])
end, { nargs = "*" })
-- vim.api.nvim_command(
--   "command! -nargs=2 WatchFile call luaeval('Watch_File(_A, _B)', expand('<args>'))")

---@param T table
local function tablelength(T)
  local count = 0
  for _ in pairs(T) do count = count + 1 end
  return count
end

--- @param opts { nickname: string, client_id: string, oauth_port: string, chat_log_path: string }
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
  --- @param opts { args: string }
  vim.api.nvim_create_user_command("TwitchView", function(opts)
    local args = splitString(opts.args or "", " ")
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
  end, { nargs = "*" })

  vim.api.nvim_create_user_command("TwitchOAuth", function()
    vim.rpcnotify(Twitch_JobId, Twitch_Oauth)
  end, { nargs = "?" })

  ---@param opts { args: string }
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
--- @param opts { nickname: string, client_id: string, oauth_port: string, chat_log_path: string, auto_start: boolean }
function Connect(opts)
  InitTwitchRpc()

  if Twitch_JobId == 0 then
    print("Twitch: cannot start rpc process")
  elseif Twitch_JobId == -1 then
    print("Twitch: rpc process is not executable")
  end

  if Twitch_JobId and opts.nickname and opts.client_id and opts.oauth_port and
      opts.chat_log_path then
    twitch_init(opts)
  end
end

--- @param opts { nickname: string, client_id: string, oauth_port: string, chat_log_path: string, auto_start: boolean }
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

  if opts.auto_start then Connect(opts) end
  ConfigureCommands()

  vim.api.nvim_create_user_command("TwitchSetup",
    function() Connect(opts) end, { nargs = "?" })
end

return MyTable
