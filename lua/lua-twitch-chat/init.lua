local function setup()
  print("setup")
end

local function test()
  print("test")
end

local table = {}
table.test = test
table.setup = setup

return table
