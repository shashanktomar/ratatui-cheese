Output list.gif
Set Theme "Catppuccin Mocha"
Set FontSize 32
Set Width 1200
Set Height 1400
Set Framerate 30
Set Padding 20
Hide
Type "cargo build --example list 2>/dev/null"
Enter
Sleep 5s
Type "cargo run --example list 2>/dev/null"
Enter
Sleep 1s
Show

# Demo 1: Styled Header (two-line items)
Sleep 1.5s

# Navigate down
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s

# Page forward
Type "l"
Sleep 1s

# Navigate a bit
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s

Sleep 1s

# Switch to Demo 2: Detail Cards
Type "n"
Sleep 1.5s

# Navigate and favorite some items
Type "j"
Sleep 0.4s
Type "f"
Sleep 0.5s
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s
Type "f"
Sleep 0.5s

# Scroll down more
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s

# Page forward
Type "l"
Sleep 1s

# Navigate on page 2
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s

Sleep 1s

# Switch to Demo 3: With Line Separator
Type "n"
Sleep 1.5s

# Navigate down
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s

# Page forward
Type "l"
Sleep 1s

# Navigate
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s

Sleep 1s

# Switch to Demo 4: Dense
Type "n"
Sleep 1.5s

# Navigate down
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s

# Page forward
Type "l"
Sleep 1s

# Navigate
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s

Sleep 1.5s

Type "q"
