local function setup()
  print("setup")
end

local function test()
  print("test")
end

local myTable = {}
myTable.test = test
myTable.setup = setup

return myTable
