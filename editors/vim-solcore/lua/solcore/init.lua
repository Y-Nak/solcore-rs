local M = {}

local defaults = {
  name = "solcore-lsp",
  root_markers = { "solcore.toml", ".git" },
  settings = {},
}

local function normalize_cmd(cmd)
  if type(cmd) == "string" then
    return { cmd }
  end
  return cmd
end

local function default_cmd()
  local compiler = vim.env.SOLCORE_PATH
  if compiler ~= nil and compiler:match("%S") then
    return { compiler, "lsp" }
  end
  local from_env = vim.env.SOLCORE_LSP_SERVER
  if from_env ~= nil and from_env:match("%S") then
    return { from_env }
  end
  local windows = vim.fn.has("win32") == 1
  local name = windows and "solcore.exe" or "solcore"
  local home = vim.fn.expand("~")
  local cargo = vim.env.CARGO_HOME
  if cargo == nil or cargo == "" then cargo = home .. "/.cargo" end
  local directories = { cargo .. "/bin", home .. "/.local/bin" }
  for directory in string.gmatch(vim.env.PATH or "", windows and "[^;]+" or "[^:]+") do
    table.insert(directories, directory)
  end
  for _, directory in ipairs(directories) do
    -- Never resolve an empty/relative PATH entry against the workspace.
    local absolute = directory:sub(1, 1) == "/"
      or (windows and (directory:match("^%a:[/\\]") or directory:match("^\\\\")))
    local candidate = directory .. "/" .. name
    if absolute and vim.fn.executable(candidate) == 1 then
      return { candidate, "lsp" }
    end
  end
  return nil
end

local function executable(cmd)
  return type(cmd) == "table" and type(cmd[1]) == "string" and vim.fn.executable(cmd[1]) == 1
end

local function buf_dir(bufnr)
  local name = vim.api.nvim_buf_get_name(bufnr)
  if name == "" then
    return vim.fn.getcwd()
  end
  return vim.fn.fnamemodify(name, ":p:h")
end

local function find_root(bufnr, opts)
  if type(opts.root_dir) == "function" then
    return opts.root_dir(vim.api.nvim_buf_get_name(bufnr), bufnr)
  end
  if type(opts.root_dir) == "string" then
    return opts.root_dir
  end

  if vim.fs and vim.fs.root then
    local root = vim.fs.root(bufnr, opts.root_markers or defaults.root_markers)
    if root then
      return root
    end
  end

  if vim.fs and vim.fs.find then
    local found = vim.fs.find(opts.root_markers or defaults.root_markers, {
      upward = true,
      path = buf_dir(bufnr),
    })[1]
    if found then
      return vim.fs.dirname(found)
    end
  end

  return buf_dir(bufnr)
end

function M.command(opts)
  opts = opts or {}
  local explicit = normalize_cmd(opts.cmd or opts.command)
  if explicit then return explicit end
  if opts.path and opts.path ~= "" then return { opts.path, "lsp" } end
  return default_cmd()
end

function M.start(opts)
  opts = vim.tbl_deep_extend("force", defaults, opts or {})
  local bufnr = opts.bufnr or vim.api.nvim_get_current_buf()
  local cmd = M.command(opts)

  if not executable(cmd) then
    vim.notify(
      string.format("Solcore LSP server not found: %s. Install Solcore or set setup({ path = '/path/to/solcore' }).", cmd and cmd[1] or "solcore"),
      vim.log.levels.WARN
    )
    return nil
  end

  local config = {
    name = opts.name,
    cmd = cmd,
    root_dir = find_root(bufnr, opts),
    settings = opts.settings,
    capabilities = opts.capabilities,
    on_attach = opts.on_attach,
  }

  if opts.init_options ~= nil then
    config.init_options = opts.init_options
  end
  if opts.handlers ~= nil then
    config.handlers = opts.handlers
  end

  return vim.lsp.start(config, {
    bufnr = bufnr,
    reuse_client = function(client, candidate)
      return client.name == candidate.name and client.config.root_dir == candidate.root_dir
    end,
  })
end

function M.setup(opts)
  opts = opts or {}
  local group = vim.api.nvim_create_augroup("solcore_lsp", { clear = true })

  vim.api.nvim_create_autocmd("FileType", {
    group = group,
    pattern = "solcore",
    callback = function(args)
      local start_opts = vim.tbl_deep_extend("force", opts, { bufnr = args.buf })
      M.start(start_opts)
    end,
  })

  if opts.start_current ~= false and vim.bo.filetype == "solcore" then
    M.start(opts)
  end
end

return M
