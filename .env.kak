def notes-today -docstring "open today's daily note" -override %{
  eval %sh{
    printf 'edit %s.notes' "$(date +%y-%m-%d)"
  }
}

def notes-previous -docstring "open previous note" -override %{
  edit %sh{
    printf '%s\n%s\n' "$(ls)" "$kak_bufname" | \
    sort | \
    uniq | \
    grep -B 1 -A 1 `basename $kak_bufname` | \
    head -n 1
  }
}

def notes-next -docstring "open next note" -override %{
  edit %sh{
    printf '%s\n%s\n' "$(ls)" "$kak_bufname" | \
    sort | \
    uniq | \
    grep -B 1 -A 1 `basename $kak_bufname` | \
    tail -n 1
  }
}

def notes-follow -docstring "open note for the tag under cursor" -override %{
  try %{
    exec ';<a-a>['
    try %{
      exec 's([^>[\]]+)>([^>[\]]+)<ret>'
      eval %sh{
        printf 'edit "%s.notes"\n' "$kak_reg_1"
        printf 'exec "gg/\[%s\]<ret>"\n' "$kak_reg_2"
      }
    } catch %{
      exec 's>([^>[\]]+)<ret>'
      eval %sh{
        printf 'exec "gg/\[%s\]<ret>"\n' "$kak_reg_1"
      }
    } catch %{
      exec 's([^>[\]]+)<ret>'
      eval %sh{
        printf 'edit "%s.notes"\n' "$kak_reg_1"
      }
    }
  }
}

def notes-backlinks -docstring "find all links to the open note" -override %{
  eval %sh{
    path="$kak_buffile"
    path="${path%%.*}"
    path="${path##$PWD/}"
    printf 'grep \\[%s(?:>[^\]]+)?\\]' "$path"
  }
}

def notes-wrap-blocks -docstring "wrap all blocks in the note" -override %{
  eval %sh{
    wrap_indent() {
      indent="^${1}(?! )"
      blocks='(?!# )(?!- )(?!\d. )'
      cmd='fmt -w 80'
      regex="(${indent}[^\n]*\n)(${indent}${blocks}[^\n]*\n)*"
      keys="%<a-s><a-i>p<a-K>\`\`\`<ret>s${regex}<ret>|${cmd}<ret>"
      echo "try %{ exec -draft '${keys}' }"
    }
    wrap_indent ''
    wrap_indent '  '
    wrap_indent '    '
    wrap_indent '      '
    wrap_indent '        '
    wrap_indent '          '
    wrap_indent '            '
    wrap_indent '              '
  }
}

def notes-mode-enable -docstring "enable notes mode for window" -override %{
  hook -group notes-hooks window NormalKey <ret> notes-follow
  hook -group notes-hooks window NormalKey <backspace> notes-backlinks

  hook -group notes-hooks window BufWritePre .* notes-wrap-blocks

  addhl window/notes regions

  addhl window/notes/raw region '^ *```$' '^ *```$' group

    addhl window/notes/raw/content fill green

    addhl window/notes/raw/delimiters regex '```' 0:bright-black

  addhl window/notes/default default-region group

    addhl window/notes/default/header regex '^ *# ' 0:yellow

    addhl window/notes/default/list-bulleted regex '^ *- '    0:bright-black
    addhl window/notes/default/list-numbered regex '^ *\d+. ' 0:bright-black

    addhl window/notes/default/inline-emphasis regex '\*([^*]+)\*' 0:bright-black+i 1:bright-white+i
    addhl window/notes/default/inline-raw      regex '`([^`]+)`'   0:bright-black 1:green

    addhl window/notes/default/link-page regex '\[([^>\]]+)\]'           0:bright-black 1:blue
    addhl window/notes/default/link-deep regex '\[([^>\]]+)>([^>\]]+)\]' 0:bright-black 1:blue 2:green
    addhl window/notes/default/link-self regex '\[>([^>\]]+)\]'           0:bright-black 1:green

    addhl window/notes/default/tag-todo regex '\[(TODO)\]' 0:bright-black 1:cyan
    addhl window/notes/default/tag-done regex '\[(DONE)\]' 0:bright-black 1:green
    addhl window/notes/default/tag-push regex '\[(PUSH)\]' 0:bright-black 1:magenta
    addhl window/notes/default/tag-skip regex '\[(SKIP)\]' 0:bright-black 1:red
    addhl window/notes/default/tag-high regex '\[(!)\]'    0:bright-black 1:red
}

def notes-mode-disable -docstring "disable notes mode for window" -override %{
  rmhooks window notes-hooks
  rmhl window/notes
}

def notes-mode-reload -docstring "reload notes mode for window" -override %{
  notes-mode-disable
  source .env.kak
  notes-mode-enable
}

rmhooks global notes-hooks

hook -group notes-hooks global BufCreate .*\.notes$ %{
  set-option buffer filetype notes
}

hook -group notes-hooks global WinSetOption filetype=notes %{
  hook -once -always window WinSetOption filetype=.* notes-mode-disable
  notes-mode-enable
}
