Output input.gif
Set Theme "Catppuccin Mocha"
Set FontSize 32
Set Width 2400
Set Height 700
Set Framerate 30
Set Padding 20
Hide
Type "cargo build --example input 2>/dev/null"
Enter
Sleep 5s
Type "cargo run --example input 2>/dev/null"
Enter
Sleep 1s
Show

# Demo 1: Basic — type a name, validate
Sleep 1s
Type "Ada Lovelace"
Sleep 0.8s
Enter
Sleep 0.8s

# Demo 2: No description — email without @, validate to show error, fix it
Tab
Sleep 1s
Type "alice"
Sleep 0.5s
Enter
Sleep 0.8s
Type "@example.com"
Sleep 0.5s
Enter
Sleep 0.8s

# Demo 3: Custom prompt — show the → prompt
Tab
Sleep 1s
Type "dark matter"
Sleep 1s

# Demo 4: Password mode — type hidden chars, validate too short then long enough
Tab
Sleep 1s
Type "ab"
Sleep 0.3s
Enter
Sleep 0.8s
Type "cdef"
Sleep 0.3s
Enter
Sleep 0.8s

# Demo 5: With value — edit pre-filled value
Tab
Sleep 1s
Backspace 6
Sleep 0.3s
Type "widget"
Sleep 0.8s

# Demo 6: Validation — starts with invalid email, fix it
Tab
Sleep 1s
Backspace 12
Sleep 0.3s
Type "ada@example.com"
Sleep 0.5s
Enter
Sleep 0.8s

# Demo 7: Live validation — ISBN, watch error update per keystroke
Tab
Sleep 1s
Type "9"
Sleep 0.2s
Type "7"
Sleep 0.2s
Type "8"
Sleep 0.2s
Type "0"
Sleep 0.2s
Type "1"
Sleep 0.2s
Type "4"
Sleep 0.2s
Type "0"
Sleep 0.2s
Type "4"
Sleep 0.2s
Type "4"
Sleep 0.2s
Type "9"
Sleep 0.2s
Type "3"
Sleep 0.2s
Type "3"
Sleep 0.2s
Type "1"
Sleep 0.8s

# Demo 8: Char limit — type 5 chars, try 6th (blocked)
Tab
Sleep 1s
Type "12345"
Sleep 0.5s
Type "6"
Sleep 0.5s
Enter
Sleep 1s

Escape
