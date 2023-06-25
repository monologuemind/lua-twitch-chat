local function getOperatingSystem()
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
---@return string
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

---@param T table
---@return integer
local function tablelength(T)
  local count = 0
  for _ in pairs(T) do count = count + 1 end
  return count
end

return {
  tablelength = tablelength,
  splitString = splitString,
  stringToBinary = stringToBinary,
  getOperatingSystem = getOperatingSystem
}
