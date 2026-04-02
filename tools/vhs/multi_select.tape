Output multi_select.gif
Set Theme "Catppuccin Mocha"
Set FontSize 32
Set Width 2400
Set Height 700
Set Framerate 30
Set Padding 20
Hide
Type "cargo build --example multi_select 2>/dev/null"
Enter
Sleep 5s
Type "cargo run --example multi_select 2>/dev/null"
Enter
Sleep 1s
Show

# Demo 1: Basic — toggle some instruments
Sleep 1s
Type "j"
Sleep 0.3s
Type " "
Sleep 0.5s
Type "j"
Sleep 0.3s
Type " "
Sleep 0.5s
Type "j"
Sleep 0.3s
Type "k"
Sleep 0.3s

Sleep 1s

# Demo 2: With limit
Type "l"
Sleep 1s

Type " "
Sleep 0.4s
Type "j"
Sleep 0.3s
Type " "
Sleep 0.4s
Type "j"
Sleep 0.3s
Type " "
Sleep 0.4s
# Try to select a 4th — should be blocked by limit
Type "j"
Sleep 0.3s
Type " "
Sleep 0.5s

Sleep 1s

# Demo 3: Pre-selected
Type "l"
Sleep 1s

# Deselect one
Type "j"
Sleep 0.3s
Type " "
Sleep 0.5s

# Select another
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type " "
Sleep 0.5s

Sleep 1s

# Demo 4: Disabled options
Type "l"
Sleep 1s

Type "j"
Sleep 0.3s
Type " "
Sleep 0.4s
# Navigate skips disabled
Type "j"
Sleep 0.3s
Type " "
Sleep 0.5s

Sleep 1s

# Demo 5: Validation
Type "l"
Sleep 1s

# Validate with nothing selected — error
Enter
Sleep 0.8s

# Select one — success
Type " "
Sleep 0.5s
Type "j"
Sleep 0.3s
Type " "
Sleep 0.5s

Sleep 1.5s

Escape
