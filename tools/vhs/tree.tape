Output tree.gif
Set Theme "Catppuccin Mocha"
Set FontSize 32
Set Width 2400
Set Height 2800
Set Framerate 30
Set Padding 20
Hide
Type "cargo build --example tree 2>/dev/null"
Enter
Sleep 5s
Type "cargo run --example tree 2>/dev/null"
Enter
Sleep 1s
Show

# Navigate down through items
Sleep 1s
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s

# Expand a folder
Type "l"
Sleep 1s

# Navigate into children
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s

# Collapse it
Type "h"
Sleep 1s

# Navigate down more
Type "j"
Sleep 0.4s
Type "j"
Sleep 0.4s

# Expand all
Type "L"
Sleep 2s

# Scroll down through everything
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s
Type "j"
Sleep 0.3s

# Collapse all
Sleep 0.5s
Type "H"
Sleep 1.5s

# Cycle count mode
Type "c"
Sleep 1s
Type "c"
Sleep 1s
Type "c"
Sleep 1s

# Toggle help
Type "?"
Sleep 2s
Type "?"
Sleep 1s

Type "q"
