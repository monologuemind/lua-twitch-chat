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
Target_Application = '/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/sample/target/debug/sample'

local function configureCommands()
  vim.api.nvim_create_user_command("TwitchTest", function()
    print("testing")

    vim.rpcnotify(Twitch_JobId, Twitch_Test)
    print("end")
  end, {})
  vim.api.nvim_create_user_command("TwitchInit", function(opts)
    local args = splitString(opts.args or "", " ")

    if tablelength(args) ~= 3 then
      vim.notify("TwitchInit requires only 3 arguments: nickname client_id port", vim.log.levels.ERROR)
      return
    end

    vim.rpcnotify(Twitch_JobId, Twitch_Init, args)
  end, { nargs = "?" })

  vim.api.nvim_create_user_command("TwitchOAuth", function()
    vim.rpcnotify(Twitch_JobId, Twitch_Oauth)
  end, { nargs = "?" })

  vim.api.nvim_create_user_command("TwitchJoin", function(opts)
    local args = splitString(opts.args or "", " ")

    if tablelength(args) == 0 then
      vim.notify("No arguments passed", vim.log.levels.ERROR)
      return
    end

    for _, value in pairs(args) do
      print(value)
    end
  end, { nargs = "?" })
end
-- Initialize RPC
function InitRpc()
  if Twitch_JobId == 0 then
    local jobid = vim.fn.jobstart({ Target_Application }, { rpc = true })
    return jobid
  else
    return Twitch_JobId
  end
end

-- Entry point. Initialize RPC. If it succeeds, then attach commands to the `rpcnotify` invocations.
function Connect()
  local id = InitRpc()

  if id == 0 then
    print("Twitch: cannot start rpc process")
  elseif id == -1 then
    print("Twitch: rpc process is not executable")
  else
    -- Mutate our jobId variable to hold the channel ID
    print(Twitch_JobId)
    Twitch_JobId = id
    print(id)

    configureCommands()
  end
end

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
