-- Recreates `kou.ppap` storyboard from Lab cache.
-- Source: storyboard.json
-- Run: cargo run -p cytoid-sb-cli -- check examples/kou_ppap/storyboard.lua

sb.controller { scanline_opacity = 0, time = 0 }

local ctrl_6 = {
  { time = 0 },
  { note_ring_color = "#FFFFFF", add_time = 0.1, note_fill_colors = { "#d7a743", "#d7a743", "#d7a743", "#d7a743", "#d7a743", "#d7a743", "#d7a743", "#d7a743", "#d7a743", "#d7a743", "#d7a743", "#d7a743" } },
}
local ctrl_6_handle = sb.controller {}
for _, kf in ipairs(ctrl_6) do
  local patch = {}
  for k, v in pairs(kf) do
    if k ~= "time" and k ~= "add_time" then patch[k] = v end
  end
  if kf.add_time then
    ctrl_6_handle:rel(kf.add_time, patch)
  else
    local t = kf.time or 0
    ctrl_6_handle:key(t, patch)
  end
end

local sprite_24 = sb.sprite { path = "ppap1.png", width = 800, height = 600, layer = 0, opacity = 1, time = 0.1, easing = "none" }
sprite_24:key(10000, { opacity = 1, destroy = true })
