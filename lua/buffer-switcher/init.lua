local M = {}

local hl = require("buffer-switcher.highlight")
local ls = require("buffer-switcher.list")
local rpc = require("buffer-switcher.rpc")
local ui = require("buffer-switcher.ui")

local api = vim.api

local default_opts = {
    border = {
        hl_group = "FloatBorder",
    },

    keymaps = {
        global = {},
        input = {},
    },
}

local config = {
    keymaps = {
        input = {},
    },
}

local function define_keymaps_wrap(args, default_opts)
    local opts = vim.tbl_deep_extend("force", vim.deepcopy(default_opts), args[4] or {})

    local rhs = args[3]
    if type(rhs) == 'string' and M.fn[rhs] then
        vim.keymap.set(args[1], args[2], M.fn[rhs], opts)
    else
        vim.keymap.set(args[1], args[2], rhs, opts)
    end
end

local set_keymaps = {
    input = function(buf)
        for _, args in ipairs(config.keymaps.input) do
            define_keymaps_wrap(args, { buffer = buf, silent = true })
        end
    end,
}

function M.define_keymaps(keymaps)
    if not keymaps then return end

    if keymaps.global then
        for _, args in ipairs(keymaps.global) do
            define_keymaps_wrap(args, { silent = true })
        end
    end

    if keymaps.input then
        for _, keymap in ipairs(keymaps.input) do
            table.insert(config.keymaps.input, keymap)
        end
    end
end

function M.setup(opts)
    if opts.border then
        ui.update_opts({ background = opts.border })
    end

    M.define_keymaps(opts.keymaps)
    ls.autocmd()
    ui.autocmd()
    hl.set_highlight_groups(opts.hl)

    rpc.register(opts.plugin_dir, opts.rpc_ns)
end

M.fn = {
    open = function()
        local buffers = ls.get_buffers()
        rpc.call.update_buffers(buffers)

        ui.open_results()
        ui.open_input(function(buf)
            local text_changed = api.nvim_create_augroup("NaughieBufferSwitcherTextChanged", { clear = true }),
            api.nvim_create_autocmd("TextChangedI", {
                group = augroup,
                buffer = buf,
                callback = function()
                    local input = ui.get_input()

                    local buffers = rpc.call.rerank(input)
                    ui.render_results(buffers)
                end,
            })

            set_keymaps.input(buf)
        end)
    end,

    open_selected_buf = ui.open_selected_buf,
    select_next = ui.select_next,
    select_prev = ui.select_prev,

    close = ui.close,
}

return M
