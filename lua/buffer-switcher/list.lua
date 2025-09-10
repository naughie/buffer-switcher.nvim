local M = {}

local mkstate = require("glocal-states")

local buffers = mkstate.tab()

local api = vim.api

function M.get_buffers()
    local current_tab_id = api.nvim_get_current_tabpage()

    local buf_current = {}
    local buf_other = {}

    for tab, buffers_in_tab in buffers.iter() do
        if tab == current_tab_id then
            for file, buf_id in pairs(buffers_in_tab) do
                table.insert(buf_current, { buf_id, file, { tab } })
            end
        else
            for file, buf_id in pairs(buffers_in_tab) do
                table.insert(buf_other, { buf_id, file, { tab } })
            end
        end
    end

    return {
        current_tab = buf_current,
        other_tabs = buf_other,
    }
end

function M.autocmd()
    local augroup = api.nvim_create_augroup("NaughieBufferSwitcherLs", { clear = true }),

    api.nvim_create_autocmd("BufEnter", {
        group = augroup,
        callback = function(ev)
            local file = ev.file
            local buf_id = ev.buf

            local listed = api.nvim_get_option_value("buflisted", { buf = buf_id })
            if not file or file == "" or not listed then return end

            local buffers_in_tab = buffers.get()
            if buffers_in_tab then
                buffers_in_tab[file] = buf_id
            else
                buffers.set({ [file] = buf_id })
            end
        end,
    })
end

return M
