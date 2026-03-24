# UI utilities module
# Shared functions for creating beautiful terminal output using gum

# Display a styled header message with border
export def header [
    message: string  # The message to display
]: nothing -> nothing {
    gum style --foreground 212 --border "rounded" --padding "0 1" $message
}

# Display a success message
export def success [
    message: string  # The message to display
]: nothing -> nothing {
    print $"(gum style --foreground 2 '✓') ($message)"
}

# Display an info message
export def info [
    message: string  # The message to display
]: nothing -> nothing {
    print $"(gum style --foreground 14 '→') ($message)"
}

# Display a warning message
export def warning [
    message: string  # The message to display
]: nothing -> nothing {
    print $"(gum style --foreground 11 '⚠') ($message)"
}

# Display an error message
export def error [
    message: string  # The message to display
]: nothing -> nothing {
    print $"(gum style --foreground 9 '✗') ($message)"
}

# Display a section title (simple, no border)
export def section [
    title: string  # The section title
]: nothing -> nothing {
    gum style --foreground 212 --bold $title
}

# Display a divider line
export def divider [
    text?: string  # Optional ghost text for the divider
]: nothing -> nothing {
    if $text == null {
        gum style --foreground 240 "─────────────────────────────────────────────"
    } else {
        gum style --foreground 240 $"────────────────── ($text) ──────────────────"
    }
}
