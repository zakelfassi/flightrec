Output assets/demo.gif

Set Theme {
  "name": "flightrec-dark",
  "black": "#101010",
  "red": "#D43B3B",
  "green": "#1A9E55",
  "yellow": "#C7860A",
  "blue": "#52524E",
  "magenta": "#B32D2D",
  "cyan": "#157A43",
  "white": "#E8E8E2",
  "brightBlack": "#2A2A28",
  "brightRed": "#D43B3B",
  "brightGreen": "#1A9E55",
  "brightYellow": "#C7860A",
  "brightBlue": "#A3A39C",
  "brightMagenta": "#9A6A08",
  "brightCyan": "#1A9E55",
  "brightWhite": "#FAFAF7",
  "background": "#101010",
  "foreground": "#E8E8E2",
  "selectionBackground": "#2A2A28",
  "cursor": "#E8E8E2"
}

Set FontFamily "IBM Plex Mono"
Set FontSize 14
Set Width 1200
Set Height 675
Set Framerate 18
Set TypingSpeed 60ms

Hide
Type "export PS1='flightrec $ '"
Enter
Type "export PATH=/Users/zakelfassi/Documents/Code/agentscope/target/release:$PATH"
Enter
Type "export FLIGHTREC_HOME=$(mktemp -d)"
Enter
Type "cd /Users/zakelfassi/Documents/Code/agentscope"
Enter
Type "echo '# agent session log' > DEMO_NOTES.md"
Enter
Sleep 100ms
Type "flightrec init --root ."
Enter
Sleep 600ms
Type "clear"
Enter
Sleep 300ms
Show

Sleep 300ms
Type "flightrec watch --once"
Enter
Sleep 3000ms

Sleep 500ms
Type "echo '## agent made changes here' >> DEMO_NOTES.md"
Enter
Sleep 800ms

Type "flightrec watch --once"
Enter
Sleep 3000ms

Sleep 500ms
Type "flightrec replay"
Enter
Sleep 2500ms

Hide
Type "rm -f DEMO_NOTES.md"
Enter
Sleep 200ms
Show
