-- A compact complex storyboard authored in Lua.
-- It mirrors patterns from real Cytoid storyboards: repeated gate sprites,
-- scanline/camera controllers, note lane overrides, text, line, video, trigger.

local gate_parts = {
  { "gate_part1.png", -624,  360, -20 },
  { "gate_part2.png",    0,  784, -10 },
  { "gate_part3.png",  624,  360,  20 },
  { "gate_part4.png",  624, -360, -20 },
  { "gate_part5.png",    0, -784,  10 },
  { "gate_part6.png", -624, -360,  20 },
}

local function make_gate_part(i, path, out_x, out_y, rot_x)
  local part = sb.sprite {
    id = "gate_part_" .. i,
    path = path,
    width = 1000,
    height = 1000,
    layer = 0,
    order = 13,
    opacity = 0,
    time = 7.783,
    y = 0,
    rot_x = rot_x,
  }

  part:key(8.200, { opacity = 1, easing = "out_quint" })
  part:key(41.513, { opacity = 1, scale = 1.2, rot_x = rot_x, easing = "out_quad" })
  part:key(42.810, { opacity = 1, scale = 1.2, rot_x = 0, x = 0, y = 0 })
  part:key(75.891, { opacity = 1, x = 0, y = 0 })
  part:key(77.189, { opacity = 1, x = out_x, y = out_y, easing = "out_quint" })
  part:key(141.053, { opacity = 1, x = out_x, y = out_y })
  part:key(143.999, { opacity = 1, x = 0, y = 0, easing = "in_out_quint" })
end

for i, part in ipairs(gate_parts) do
  make_gate_part(i, part[1], part[2], part[3], part[4])
end

local camera = sb.controller { id = "camera_fx" }
camera:key(0.0, {
  background_dim = 1,
  note_opacity_multiplier = 0,
  scanline_opacity = 0,
  ui_opacity = 0,
})
camera:key(6.461532, { scanline_opacity = 1, ui_opacity = 1 })
camera:key(7.461532, { scanline_color = "#DE595DFF" })
camera:key(11.461532, { scanline_color = "#FFFFFFFF" })
camera:key(36.999964, { note_ring_color = "#7AD6B3FF" })
camera:key(51.769180, { note_ring_color = "#DE595DFF" })
camera:key(66.461472, { override_scanline_pos = true, scanline_pos = 0.0 })
camera:key(67.153779, { override_scanline_pos = true, scanline_pos = 0.75 })
camera:key(68.307624, { override_scanline_pos = true, scanline_pos = 0.0 })

local title = sb.text {
  id = "section_title",
  text = "GATE",
  time = 6.2,
  x = 0,
  y = 210,
  size = 48,
  opacity = 0,
}
title:key(6.6, { opacity = 1, easing = "out_quint" })
title:key(8.2, { opacity = 0, easing = "in_quint" })

local rail = sb.line {
  id = "gate_rail",
  time = 7.4,
  opacity = 0,
  width = 6,
  color = "#FFFFFFFF",
  pos = {
    { x = -0.65, y = 0.0 },
    { x = 0.65, y = 0.0 },
  },
}
rail:key(7.8, { opacity = 1 })
rail:key(8.6, { opacity = 0 })

local mv = sb.video {
  id = "bg_video",
  path = "gate_loop.mp4",
  time = 35.9,
  opacity = 0,
  width = 1000,
  height = 600,
  layer = -1,
}
mv:key(36.2, { opacity = 0.45 })
mv:key(41.0, { opacity = 0 })

local lane_overrides = {
  { note = 422, y = 0.0 },
  { note = 423, y = 0.0 },
  { note = 424, y = 0.75 },
  { note = 425, y = 0.75 },
  { note = 426, y = 0.0 },
  { note = 427, y = 0.0 },
  { note = 428, y = 0.6875 },
  { note = 429, y = 0.6875 },
  { note = 430, y = 0.0 },
  { note = 431, y = 0.0 },
  { note = 432, y = 0.625 },
  { note = 433, y = 0.625 },
}

for _, nc in ipairs(lane_overrides) do
  sb.note_controller {
    note = nc.note,
    time = 0,
    override_y = true,
    y = nc.y,
  }
end

sb.trigger {
  type = "NoteClear",
  notes = { 422, 424, 428 },
  spawn = { "section_title" },
  destroy = { "gate_rail" },
  uses = 1,
}
