#!/bin/sh
# Main executable of TermFetch Studio.app: Finder runs this on double-click and it
# opens the studio in the terminal the user works in. The studio itself sets
# everything up on first start.

here=$(cd "$(dirname "$0")" && pwd)
app=$(cd "$here/../.." && pwd)
bin="$here/termfetch-studio"

case "$app" in
    /Volumes/* | */AppTranslocation/*)
        osascript -e 'display dialog "Drag TermFetch Studio into the Applications folder first, then open it from there." with title "TermFetch Studio" buttons {"OK"} default button "OK" with icon caution' >/dev/null 2>&1
        exit 0
        ;;
esac

# The user has already allowed this app; let the terminal run the tools inside it as well.
xattr -dr com.apple.quarantine "$app" 2>/dev/null

# Nerd Font symbols before the terminal starts, so its icons render from the first frame.
for font in "$app/Contents/Resources/fonts/"*.ttf; do
    [ -f "$font" ] && [ ! -e "$HOME/Library/Fonts/$(basename "$font")" ] &&
        mkdir -p "$HOME/Library/Fonts" && cp "$font" "$HOME/Library/Fonts/"
done

# The terminal used most recently among the installed ones; Terminal.app otherwise.
terminal=Terminal
latest=""
for path in /Applications/iTerm.app /Applications/Ghostty.app /Applications/kitty.app \
    /Applications/WezTerm.app "$HOME/Applications/iTerm.app" "$HOME/Applications/Ghostty.app" \
    "$HOME/Applications/kitty.app" "$HOME/Applications/WezTerm.app" \
    /System/Applications/Utilities/Terminal.app; do
    [ -d "$path" ] || continue
    used=$(mdls -raw -name kMDItemLastUsedDate "$path" 2>/dev/null)
    case "$used" in "" | "(null)") continue ;; esac
    if [ -z "$latest" ] || [ "$used" \> "$latest" ]; then
        latest=$used
        terminal=$(basename "$path" .app)
        terminal_path=$path
    fi
done

case "$terminal" in
    Ghostty) exec open -na "$terminal_path" --args -e "$bin" ;;
    kitty) exec open -na "$terminal_path" --args "$bin" ;;
    WezTerm) exec open -na "$terminal_path" --args start -- "$bin" ;;
    iTerm) exec open -a "$terminal_path" "$app/Contents/Resources/TermFetch Studio.command" ;;
    *) exec open -a Terminal "$app/Contents/Resources/TermFetch Studio.command" ;;
esac
