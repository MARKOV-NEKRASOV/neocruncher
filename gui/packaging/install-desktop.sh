#!/bin/sh
# Install a per-user application-menu entry for this extracted GUI trial.
set -eu

APPDIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
entry_dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
entry="$entry_dir/neocruncher-gui-trial.desktop"

if [ ! -x "$APPDIR/AppRun" ] || [ ! -f "$APPDIR/neocruncher.png" ]; then
  printf 'NeoCruncher: extract the complete archive before installing the menu entry.\n' >&2
  exit 2
fi
if [ -e "$entry" ] && ! grep -q '^X-NeoCruncher-Trial=true$' "$entry"; then
  printf 'NeoCruncher: refusing to replace an unrelated desktop entry: %s\n' "$entry" >&2
  exit 2
fi

mkdir -p "$entry_dir"
{
  printf '[Desktop Entry]\n'
  printf 'Type=Application\n'
  printf 'Name=NeoCruncher GUI (trial 0.1.3)\n'
  printf 'Comment=Research frontend for high-precision mathematical constants\n'
  printf 'Exec="%s/AppRun"\n' "$APPDIR"
  printf 'Icon=%s/neocruncher.png\n' "$APPDIR"
  printf 'Categories=Science;Math;\n'
  printf 'Terminal=false\n'
  printf 'StartupNotify=true\n'
  printf 'X-NeoCruncher-Trial=true\n'
} > "$entry"
chmod 644 "$entry"

if command -v desktop-file-validate >/dev/null 2>&1; then
  desktop-file-validate "$entry"
fi
printf 'Installed menu entry: %s\n' "$entry"
printf 'Keep the extracted application at %s; moving it will break the menu entry.\n' "$APPDIR"
