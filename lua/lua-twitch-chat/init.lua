---@type { splitString: fun(name: string, delimiter: string): table; stringToBinary: fun(str: string): string; getOperatingSystem: fun(): string; tablelength: fun(T: table): integer }
local helpers = dofile("./helpers.lua")
---@type { isColorLight: fun(str: string): boolean; load_highlights: fun(data: string) }
local highlights = dofile("./highlights.lua")

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

print(vim.cmd("pwd"))

-- Just a table that we will return with some stuff on it
MyTable = {}

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
        highlights.load_highlights(data)
      end
    end
  else
    -- THIS IS SUPER CLOSE: BLOCKING  AND ONLY WORKED UNTIL IT HAD TEXT BEYOND ITSELF
    vim.schedule(function()
      vim.api.nvim_set_current_buf(bufnr)
      vim.fn.cursor({ vim.fn.line('$'), 0 })
      -- vim.cmd("silent execute 'buffer' " .. bufnr ..
      --   " | execute \"normal G\"")
      local file = io.open(hname, "r")
      if (file) then
        local data = file:read("*a")
        if (data ~= nil and string.len(data) ~= h) then
          h = string.len(data)
          highlights.load_highlights(data)
        end
      end
      vim.api.nvim_set_current_buf(current_bufnr)
      -- vim.cmd("execute 'buffer' " .. current_bufnr)
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
  local args = helpers.splitString(opts.args or "", " ")

  Watch_File(args[1], args[2])
end, { nargs = "*" })

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
  ConfigureCommands(twitch_init)

  vim.api.nvim_create_user_command("TwitchSetup",
    function() Connect(opts) end, { nargs = "?" })
end

return MyTable
