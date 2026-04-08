Output select.gif
Set Theme "Catppuccin Mocha"
Set FontSize 32
Set Width 2400
Set Height 700
Set Framerate 30
Set Padding 20
Hide
Type "cargo build --example select 2>/dev/null"
Enter
Sleep 5s
Type "cargo run --example select 2>/dev/null"
Enter
Sleep 1s
Show

# Demo 1: Basic — navigate destinations
Sleep 1s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "k"
Sleep 0.3s

Sleep 1s

# Demo 2: Custom cursor
Type "l"
Sleep 1s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s

Sleep 1s

# Demo 3: Pre-selected
Type "l"
Sleep 1s
Type "j"
Sleep 0.3s
Type "k"
Sleep 0.3s
Type "k"
Sleep 0.3s

Sleep 1s

# Demo 4: Disabled options — navigation skips disabled
Type "l"
Sleep 1s
Type "j"
Sleep 0.3s
# Skips Europa (disabled), lands on Titan
Type "j"
Sleep 0.3s

Sleep 1s

# Demo 5: Validation
Type "l"
Sleep 1s
# Starts on Europa — shows error
Sleep 0.8s
Type "j"
Sleep 0.5s
# Titan — shows success
Type "k"
Sleep 0.3s
Type "k"
Sleep 0.5s

Sleep 1.5s

Escape
