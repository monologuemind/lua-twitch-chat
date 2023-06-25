---@type { splitString: fun(name: string, delimiter: string): table; stringToBinary: fun(str: string): string; getOperatingSystem: fun(): string }
local helpers = require "./helpers.lua"

--- @param hexCode string
local function isColorLight(hexCode)
  -- Remove the '#' symbol from the beginning of the hex code
  local hex = string.sub(hexCode, 2)

  -- Convert hex to RGB values
  local r = tonumber(string.sub(hex, 1, 2), 16)
  local g = tonumber(string.sub(hex, 3, 4), 16)
  local b = tonumber(string.sub(hex, 5, 6), 16)

  -- Calculate relative luminance using the sRGB color space formula
  local relativeLuminance = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255

  -- Compare relative luminance to a threshold value (0.5) to determine if it is a light color
  return relativeLuminance > 0.5
end

--- @param data string
local load_highlights = function(data)
  ---@type { [string]: nil | string[] }
  local hex_groups = {}
  local user_colors = helpers.splitString(data, "\n")
  for _, v in pairs(user_colors) do
    local key_value_pair = helpers.splitString(v, ",")
    local group_exists = hex_groups[key_value_pair[2]]
    if group_exists == nil then hex_groups[key_value_pair[2]] = {} end
    table.insert(hex_groups[key_value_pair[2]], key_value_pair[1])
  end
  for hex_code, user_names in pairs(hex_groups) do
    local binary_code = helpers.stringToBinary(hex_code)
    local background = "#ffffff"
    if isColorLight(hex_code) then background = "#000000" end
    vim.api.nvim_set_hl(0, binary_code,
      { fg = hex_code, bg = background, bold = true })
    for _, user_name in pairs(user_names) do
      local cm1 = "syntax keyword " .. user_name .. " " .. user_name
      vim.cmd(cm1)
      local cm2 = "highlight link " .. user_name .. " " .. binary_code
      vim.cmd(cm2)
    end
  end
end

return { isColorLight = isColorLight, load_highlights = load_highlights }
