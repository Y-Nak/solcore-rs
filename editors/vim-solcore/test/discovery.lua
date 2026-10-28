-- Run from the repository root:
-- nvim --headless -u NONE -l editors/vim-solcore/test/discovery.lua
vim.opt.runtimepath:append(vim.fn.getcwd() .. "/editors/vim-solcore")
local solcore = require("solcore")
local temporary = vim.fn.tempname()
vim.fn.mkdir(temporary .. "/cargo/bin", "p")
vim.fn.mkdir(temporary .. "/path", "p")
for _, directory in ipairs({ "/cargo/bin", "/path" }) do
  local executable = temporary .. directory .. "/solcore"
  vim.fn.writefile({ "#!/bin/sh", "exit 0" }, executable)
  vim.fn.setfperm(executable, "rwx------")
end
vim.env.CARGO_HOME = temporary .. "/cargo"
vim.env.PATH = ".:relative/bin:" .. temporary .. "/path"
vim.env.SOLCORE_PATH = nil
vim.env.SOLCORE_LSP_SERVER = nil
assert(vim.deep_equal(solcore.command(), { temporary .. "/cargo/bin/solcore", "lsp" }))
vim.env.SOLCORE_LSP_SERVER = "/legacy/solcore-lsp"
assert(vim.deep_equal(solcore.command(), { "/legacy/solcore-lsp" }))
vim.env.SOLCORE_PATH = "/My Tools/solcore"
assert(vim.deep_equal(solcore.command(), { "/My Tools/solcore", "lsp" }))
assert(vim.deep_equal(solcore.command({ path = "/configured/solcore" }), { "/configured/solcore", "lsp" }))
assert(vim.deep_equal(solcore.command({ cmd = { "/explicit", "arg" } }), { "/explicit", "arg" }))

-- Exercise the Vim autoload command through Neovim's Vimscript runtime too.
assert(vim.deep_equal(vim.fn["solcore#lsp_command"](), { "/My Tools/solcore", "lsp" }))
vim.g.solcore_path = "/configured/solcore"
assert(vim.deep_equal(vim.fn["solcore#lsp_command"](), { "/configured/solcore", "lsp" }))
vim.g.solcore_lsp_command = { "/explicit", "arg" }
assert(vim.deep_equal(vim.fn["solcore#lsp_command"](), { "/explicit", "arg" }))
vim.g.solcore_lsp_command = nil
vim.g.solcore_path = nil
vim.env.SOLCORE_PATH = nil
vim.env.SOLCORE_LSP_SERVER = nil
assert(vim.deep_equal(vim.fn["solcore#lsp_command"](), { temporary .. "/cargo/bin/solcore", "lsp" }))
vim.fn.delete(temporary, "rf")
print("Solcore Neovim and Vim discovery tests passed")
