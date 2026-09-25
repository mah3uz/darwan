#!/usr/bin/env bash
assets="$(dirname "$0")/Assets"
t=$1
l=${t,,}
for c in "$t" "$l" "${l//-/_}"; do
  gif=$assets/$c.gif
  [[ -f $gif ]] || continue
  dim=${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}
  if [[ $KITTY_WINDOW_ID || $GHOSTTY_RESOURCES_DIR ]] && command -v kitten >/dev/null; then
    kitten icat --clear --transfer-mode=memory --unicode-placeholder --stdin=no --place="${dim}@0x0" "$gif" | sed '$d' | sed $'$s/$/\e[m/'
  elif command -v chafa >/dev/null; then
    chafa -s "$dim" "$gif"
  else
    echo "no image viewer (install chafa)"
  fi
  exit
done
echo "no preview for $t"
