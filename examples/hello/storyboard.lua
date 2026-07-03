local glow = sb.sprite { id = "hello_glow", path = "hello_glow.png" }
glow:key(0.0, { opacity = 0, scale = 0.8 })
glow:key(1.0, { opacity = 1, scale = 1.0, ease = "out_quint" })
