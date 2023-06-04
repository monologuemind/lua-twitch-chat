--[[
 CREATE CHAT LISTENER
  * Validate settings file (startup or or first call)
  * Create chat log file (startup)
  * Start background server
 VIEW CHAT FILE
  * Open chat log file as buffer
    * Need to update buffer as file updates come in from background server
--]]

myTable = {}
myTable.settings = {
  file = "/home/michaelbuser/Documents/git/nvim-plugins/lua-twitch-chat/socket/target/debug/socket",
  -- file = nil,
  thread = nil
}


myTable.setup = function()
  require("./job.lua")
end

return myTable
