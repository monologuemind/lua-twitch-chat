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
if not Twitch_JobId then
  Twitch_JobId = 0
end

-- Constants for RPC messages
Twitch_Oauth = 'oauth'
Twitch_Init = 'init'
Twitch_Test = 'test'

-- The path to the binary that was created out of 'cargo build' or 'cargo build --release'. This will generally be 'target/release/name'
Target_Application = '/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/target/debug/socket'

MyTable = {}
-- MyTable.settings = {
--   file = "/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/target/debug/socket",
--   -- file = nil,
--   thread = nil
-- }

function ConfigureCommands()
  vim.api.nvim_create_user_command("TwitchTest", function()
    vim.rpcnotify(Twitch_JobId, Twitch_Test)
  end, {})
  vim.api.nvim_create_user_command("TwitchInit", function(opts)
    local args = splitString(opts.args or "", " ")

    if tablelength(args) ~= 3 then
      vim.notify("TwitchInit requires only 3 arguments: nickname client_id port", vim.log.levels.ERROR)
      return
    end

    vim.rpcnotify(Twitch_JobId, Twitch_Init, args[0], args[1], args[2])
  end, { nargs = "3" })

  vim.api.nvim_create_user_command("TwitchOAuth", function()
    vim.rpcnotify(Twitch_JobId, Twitch_Oauth)
  end, { nargs = "0" })

  vim.api.nvim_create_user_command("TwitchJoin", function(opts)
    local args = splitString(opts.args or "", " ")

    if tablelength(args) == 0 then
      vim.notify("No arguments passed", vim.log.levels.ERROR)
      return
    end
  end, { nargs = "0" })
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

MyTable.setup = function(opts)
  print(opts)
  -- Setting up the exit of the editor to also stop the socket
  local twitch_group = vim.api.nvim_create_augroup("TwitchSocket", { clear = true })
  vim.api.nvim_create_autocmd("ExitPre", {
    group = twitch_group,
    callback = function()
      if Twitch_JobId then
        vim.fn.jobstop(Twitch_JobId)
      end
    end
  })

  Connect()
end

return MyTable
