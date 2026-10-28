" Command factory for Vim LSP clients such as vim-lsp.
function! solcore#lsp_command() abort
  if exists('g:solcore_lsp_command') && !empty(g:solcore_lsp_command)
    return type(g:solcore_lsp_command) == v:t_list ? copy(g:solcore_lsp_command) : [g:solcore_lsp_command]
  endif
  if exists('g:solcore_path') && !empty(g:solcore_path)
    return [g:solcore_path, 'lsp']
  endif
  if !empty($SOLCORE_PATH)
    return [$SOLCORE_PATH, 'lsp']
  endif
  if !empty($SOLCORE_LSP_SERVER)
    return [$SOLCORE_LSP_SERVER]
  endif
  let windows = has('win32') || has('win64')
  let name = windows ? 'solcore.exe' : 'solcore'
  let cargo = empty($CARGO_HOME) ? expand('~/.cargo') : $CARGO_HOME
  let directories = [cargo . '/bin', expand('~/.local/bin')] + split($PATH, windows ? ';' : ':')
  for directory in directories
    if directory !~# '^/' && !(windows && directory =~# '^\%([A-Za-z]:[/\\]\|\\\\\)')
      continue
    endif
    let candidate = directory . '/' . name
    if executable(candidate)
      return [candidate, 'lsp']
    endif
  endfor
  throw 'Solcore compiler not found; install Solcore or set g:solcore_path'
endfunction
