-- Recreates `kyr.bite_your_nails` CHAOS storyboard from Lab cache.
-- Source: CHAOS_storyboard.json (26 note_controllers + 4 camera/scanline controllers)
-- Auto-generated; regenerate: python tool/gen_lua_from_sb.py examples/levels/kyr.bite_your_nails/CHAOS_storyboard.json examples/kyr_bite_your_nails/storyboard.lua

-- Note lane overrides (data from original storyboard.json)
local note_controllers = {
  { override_y = true, y = 0.0, time = 0.0, note = 422 },
  { override_y = true, y = 0.0, time = 0.0, note = 423 },
  { override_y = true, y = 0.75, time = 0.0, note = 424 },
  { override_y = true, y = 0.75, time = 0.0, note = 425 },
  { override_y = true, y = 0.0, time = 0.0, note = 426 },
  { override_y = true, y = 0.0, time = 0.0, note = 427 },
  { override_y = true, y = 0.6875, time = 0.0, note = 428 },
  { override_y = true, y = 0.6875, time = 0.0, note = 429 },
  { override_y = true, y = 0.0, time = 0.0, note = 430 },
  { override_y = true, y = 0.0, time = 0.0, note = 431 },
  { override_y = true, y = 0.625, time = 0.0, note = 432 },
  { override_y = true, y = 0.625, time = 0.0, note = 433 },
  { override_y = true, y = 0.0, time = 0.0, note = 451 },
  { override_y = true, y = 0.0, time = 0.0, note = 452 },
  { override_y = true, y = 0.75, time = 0.0, note = 453 },
  { override_y = true, y = 0.75, time = 0.0, note = 454 },
  { override_y = true, y = 0.0, time = 0.0, note = 455 },
  { override_y = true, y = 0.0, time = 0.0, note = 456 },
  { override_y = true, y = 0.6875, time = 0.0, note = 457 },
  { override_y = true, y = 0.6875, time = 0.0, note = 458 },
  { override_y = true, y = 0.0, time = 0.0, note = 459 },
  { override_y = true, y = 0.0, time = 0.0, note = 460 },
  { override_y = true, y = 0.0, time = 0.0, note = 461 },
  { override_y = true, y = 0.625, time = 0.0, note = 462 },
  { override_y = true, y = 0.625, time = 0.0, note = 463 },
  { override_y = true, y = 0.625, time = 0.0, note = 464 },
}

for _, nc in ipairs(note_controllers) do
  sb.note_controller(nc)
end

local ctrl_38 = {
  { scanline_opacity = 1.0, time = 0.0 },
}
local ctrl_38_handle = sb.controller {}
for _, kf in ipairs(ctrl_38) do
  local t = kf.time
  local patch = {}
  for k, v in pairs(kf) do
    if k ~= "time" then patch[k] = v end
  end
  ctrl_38_handle:key(t, patch)
end

local ctrl_51 = {
  { scanline_color = "#FFFFFFFF", time = 6.461532 },
  { scanline_color = "#DE595DFF", time = 7.461532 },
  { scanline_color = "#DE595DFF", time = 10.461532 },
  { scanline_color = "#FFFFFFFF", time = 11.461532 },
  { scanline_color = "#FFFFFFFF", time = 35.999964 },
  { scanline_color = "#7AD6B3FF", time = 36.999964 },
  { scanline_color = "#7AD6B3FF", time = 39.999964 },
  { scanline_color = "#FFFFFFFF", time = 40.999964 },
  { scanline_color = "#FFFFFFFF", time = 50.76918 },
  { scanline_color = "#DE595DFF", time = 51.76918 },
  { scanline_color = "#DE595DFF", time = 54.76918 },
  { scanline_color = "#FFFFFFFF", time = 55.76918 },
  { scanline_color = "#FFFFFFFF", time = 65.538396 },
  { scanline_color = "#FCF2F2FF", time = 66.461472 },
  { scanline_color = "#F1EFEDFF", time = 67.384548 },
  { scanline_color = "#EFE3E1FF", time = 68.307624 },
  { scanline_color = "#E5E1DDFF", time = 69.2307 },
  { scanline_color = "#E4D6D3FF", time = 70.153776 },
  { scanline_color = "#7AD6B3FF", time = 71.153776 },
  { scanline_color = "#7AD6B3FF", time = 72.923004 },
  { scanline_color = "#D6CCACFF", time = 73.84608 },
  { scanline_color = "#CED5B2FF", time = 74.769156 },
  { scanline_color = "#DCCBABFF", time = 75.692232 },
  { scanline_color = "#D4D5B2FF", time = 76.615308 },
  { scanline_color = "#DDCBABFF", time = 77.538384 },
  { scanline_color = "#7AD6B3FF", time = 78.538384 },
  { scanline_color = "#7AD6B3FF", time = 80.307612 },
  { scanline_color = "#7AD6B3FF", time = 81.307612 },
  { scanline_color = "#7AD6B3FF", time = 84.307612 },
  { scanline_color = "#FFFFFFFF", time = 85.307612 },
  { scanline_color = "#FFFFFFFF", time = 87.69222 },
  { scanline_color = "#DE595DFF", time = 88.69222 },
  { scanline_color = "#DE595DFF", time = 91.69222 },
  { scanline_color = "#FFFFFFFF", time = 92.69222 },
  { scanline_color = "#FFFFFFFF", time = 98.307594 },
  { scanline_color = "#FCF2F2FF", time = 99.23067 },
  { scanline_color = "#7AD6B3FF", time = 100.23067 },
  { scanline_color = "#7AD6B3FF", time = 100.615284 },
  { scanline_color = "#D6CCACFF", time = 101.53836 },
  { scanline_color = "#7AD6B3FF", time = 102.53836 },
  { scanline_color = "#7AD6B3FF", time = 102.922974 },
  { scanline_color = "#D6CCACFF", time = 103.84605 },
  { scanline_color = "#7AD6B3FF", time = 104.84605 },
  { scanline_color = "#7AD6B3FF", time = 105.230664 },
  { scanline_color = "#DE595DFF", time = 106.230664 },
  { scanline_color = "#DE595DFF", time = 107.999892 },
  { scanline_color = "#7AD6B3FF", time = 108.999892 },
  { scanline_color = "#7AD6B3FF", time = 111.999892 },
  { scanline_color = "#FFFFFFFF", time = 112.999892 },
  { scanline_color = "#FFFFFFFF", time = 137.538324 },
  { scanline_color = "#DE595DFF", time = 138.538324 },
  { scanline_color = "#DE595DFF", time = 141.538324 },
  { scanline_color = "#FFFFFFFF", time = 142.538324 },
}
local ctrl_51_handle = sb.controller {}
for _, kf in ipairs(ctrl_51) do
  local t = kf.time
  local patch = {}
  for k, v in pairs(kf) do
    if k ~= "time" then patch[k] = v end
  end
  ctrl_51_handle:key(t, patch)
end

local ctrl_116 = {
  { ui_opacity = 1.0, time = 0.0 },
}
local ctrl_116_handle = sb.controller {}
for _, kf in ipairs(ctrl_116) do
  local t = kf.time
  local patch = {}
  for k, v in pairs(kf) do
    if k ~= "time" then patch[k] = v end
  end
  ctrl_116_handle:key(t, patch)
end

local ctrl_129 = {
  { override_scanline_pos = true, scanline_pos = 0.0, time = 66.461472 },
  { override_scanline_pos = false, scanline_pos = 0.75, time = 67.153779, comment = "is_60_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.75, time = 67.153779 },
  { override_scanline_pos = false, scanline_pos = 0.0, time = 68.307624, comment = "is_61_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.0, time = 68.307624 },
  { override_scanline_pos = false, scanline_pos = 0.6875, time = 68.999931, comment = "is_62_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.6875, time = 68.999931 },
  { override_scanline_pos = false, scanline_pos = 0.0, time = 70.153776, comment = "is_63_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.0, time = 70.153776 },
  { override_scanline_pos = false, scanline_pos = 0.625, time = 70.846083, comment = "is_64_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.625, time = 70.846083 },
  { override_scanline_pos = false, scanline_pos = 0.0, time = 71.999928, comment = "is_65_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.0, time = 73.84608 },
  { override_scanline_pos = false, scanline_pos = 0.75, time = 74.538387, comment = "is_68_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.75, time = 74.538387 },
  { override_scanline_pos = false, scanline_pos = 0.0, time = 75.692232, comment = "is_69_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.0, time = 75.692232 },
  { override_scanline_pos = false, scanline_pos = 0.6875, time = 76.384539, comment = "is_70_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.6875, time = 76.384539 },
  { override_scanline_pos = false, scanline_pos = 0.0, time = 77.538384, comment = "is_71_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.0, time = 77.538384 },
  { override_scanline_pos = false, scanline_pos = 0.625, time = 78.230691, comment = "is_72_in-page" },
  { override_scanline_pos = true, scanline_pos = 0.625, time = 78.230691 },
  { override_scanline_pos = false, scanline_pos = 0.0, time = 79.384536, comment = "is_73_in-page" },
}
local ctrl_129_handle = sb.controller {}
for _, kf in ipairs(ctrl_129) do
  local t = kf.time
  local patch = {}
  for k, v in pairs(kf) do
    if k ~= "time" then patch[k] = v end
  end
  ctrl_129_handle:key(t, patch)
end
