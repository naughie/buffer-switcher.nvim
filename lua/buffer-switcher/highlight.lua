local M = {}

local api = vim.api

local ns = api.nvim_create_namespace("NaughieBufferSwitcherHl")

local default_hl = {
    cursor = { link = "CursorLine" },
    matched = { link = "Search" },
    frame = { link = "FloatBorder" },
    frame_title = { link = "Normal" },
}

local hl_names = {
    cursor = "BufferSwitcherCursor",
    matched = "BufferSwitcherMatched",
    frame = "BufferSwitcherFrame",
    frame_title = "BufferSwitcherFrameTitle",
}

function M.set_highlight_groups(opts)
    for key, hl in pairs(hl_names) do
        if opts and opts[key] then
            api.nvim_set_hl(0, hl, opts[key])
        elseif default_hl[key] then
            api.nvim_set_hl(0, hl, default_hl[key])
        end
    end
end

M.set_extmark = {}

for key, hl in pairs(hl_names) do
    M.set_extmark[key] = function(buf, args)
        if args.virt_text then
            local opts = {
                virt_text = { { args.virt_text, hl } },
                virt_text_pos = args.pos,
            }

            return api.nvim_buf_set_extmark(buf, ns, args.line, args.col, opts)
        else
            local opts = {
                end_row = args.line,
                end_col = args.end_col,
                hl_group = hl,
            }
            if args.hl_eol then opts.hl_eol = true end

            return api.nvim_buf_set_extmark(buf, ns, args.line, args.start_col, opts)
        end
    end
end

M.delete_extmark = function(buf, ext_id)
    api.nvim_buf_del_extmark(buf, ns, ext_id)
end

M.clear_extmarks = function(buf)
    api.nvim_buf_clear_namespace(buf, ns, 0, -1)
end

return M

