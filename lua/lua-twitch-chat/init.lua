--[[
 CREATE CHAT LISTENER
  * Validate settings file (startup or or first call)
  * Create chat log file (startup)
  * Start background server
 VIEW CHAT FILE
  * Open chat log file as buffer
    * Need to update buffer as file updates come in from background server
--]]
-- TODO: Figure out if we want to have settings here
-- Perhaps they would only be for the audio/video functionality
-- local settings = {
--   twitch_client_id = nil
--   twitch_client_secret = nil
-- }

local myTable = {}
myTable.settings = {
  file = ""
}

local function test()
  print(myTable.settings.file)
  local file = io.open("init.lua", "r")
  print(file)
  if file then
    local f = file:read("*a")
    print("f", f)
  end
  print("test")
end

local api = vim.api
local buf, win
local function open()
  buf = api.nvim_create_buf(false, true) -- create new emtpy buffer

  api.nvim_buf_set_option(buf, 'bufhidden', 'wipe')

  -- get dimensions
  local width = api.nvim_get_option("columns")
  local height = api.nvim_get_option("lines")

  -- calculate our floating window size
  local win_height = math.ceil(height * 0.8 - 4)
  local win_width = math.ceil(width * 0.8)

  -- and its starting position
  local row = math.ceil((height - win_height) / 2 - 1)
  local col = math.ceil((width - win_width) / 2)

  -- set some options
  local opts = {
    style = "minimal",
    relative = "editor",
    width = win_width,
    height = win_height,
    row = row,
    col = col
  }

  -- and finally create it with buffer attached
  win = api.nvim_open_win(buf, true, opts)
end

test()

myTable.test = test
myTable.open = open


return myTable
