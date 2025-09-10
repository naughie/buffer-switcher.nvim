local M = {}

local hl = require("buffer-switcher.highlight")

local mkstate = require("glocal-states")
local myui = require("my-ui")

local states = mkstate.tab()

local api = vim.api

local frame = {
    padding = 3,
    hor = "─",
    vert = "│",
    corners = { "╮", "╭", "╰", "╯" },

    titles = {
        " Current Tab ",
        " Other Tabs ",
    },
}
local frame_len = {
    hor = {
        len = string.len(frame.hor),
        width = vim.fn.strwidth(frame.hor),
    },
    vert = {
        len = string.len(frame.vert),
        width = vim.fn.strwidth(frame.vert),
    },

    titles = {
        max_width = (function()
            local max_width = 0
            for _, title in ipairs(frame.titles) do
                max_width = math.max(max_width, vim.fn.strwidth(title))
            end
            return max_width
        end)(),
    },
}

local ui = myui.declare_ui({
    main = { close_on_companion_closed = true },
    geom = {
        main = {
            width = function() return math.floor(api.nvim_get_option("columns") * 0.25) end,
            height = function() return math.floor(api.nvim_get_option("lines") * 0.4) end,
        },
        companion = {
            width = function() return math.floor(api.nvim_get_option("columns") * 0.25) end,
            height = 1,
        },
    },
})

local function render_buf_item(buf_item, start_line, max_width)
    local ext = {}

    local file_path = buf_item[2]

    local padding = frame.padding
    local left_pad = string.rep(" ", padding)
    local right_pad = string.rep(" ", padding + max_width - vim.fn.strwidth(file_path))

    local line = string.format("%s%s%s%s%s", frame.vert, left_pad, file_path, right_pad, frame.vert)

    local matched = buf_item[4]

    if matched then
        local left_pad_len = padding + frame_len.vert.len

        for _, range in ipairs(matched) do
            local start_idx = range.start_idx + left_pad_len
            local end_idx = range.end_idx + left_pad_len

            table.insert(ext, {
                start_col = start_idx,
                end_col = end_idx,
                line = start_line,
                hl = "matched",
            })
        end
    end

    table.insert(ext, {
        start_col = 0,
        end_col = frame_len.vert.len,
        line = start_line,
        hl = "frame",
    })
    local line_len = string.len(line)
    table.insert(ext, {
        start_col = line_len - frame_len.vert.len,
        end_col = line_len,
        line = start_line,
        hl = "frame",
    })

    return line, ext
end

local function render_items(items, lines, exts, current_states, title)
    local title_virt = {
        virt_text = title,
        pos = "overlay",
        line = #lines,
        col = frame_len.vert.len + frame_len.hor.len * math.ceil(frame.padding / frame_len.hor.width),
        hl = "frame_title",
    }
    table.insert(exts, { title_virt })

    local hor_total_width = current_states.max_width + 2 * frame.padding
    local frame_hor_rep = math.floor(hor_total_width / frame_len.hor.width)
    local frame_hor = string.rep(frame.hor, frame_hor_rep)
    local frame_hor_len = frame_len.hor.len * frame_hor_rep + 2 * frame_len.vert.len

    local top_frame_ext = {
        start_col = 0,
        end_col = frame_hor_len,
        line = #lines,
        hl = "frame",
    }
    table.insert(lines, frame.corners[2] .. frame_hor .. frame.corners[1])

    for _, buf_item in ipairs(items) do
        local line, ext = render_buf_item(buf_item, #lines, current_states.max_width)
        table.insert(lines, line)
        table.insert(exts, ext)
    end

    local bottom_frame_ext = {
        start_col = 0,
        end_col = frame_hor_len,
        line = #lines,
        hl = "frame",
    }
    table.insert(lines, frame.corners[3] .. frame_hor .. frame.corners[4])

    table.insert(exts, { top_frame_ext, bottom_frame_ext })

    return lines, exts
end

local function update_states(buffers)
    local current_states = states.get()
    local calc_width = not current_states or current_states.max_width == nil

    local current_tab = {}
    local other_tabs = {}

    local max_width = 0

    for _, buf_item in ipairs(buffers.current_tab) do
        local matched = buf_item[4] ~= nil and #buf_item[4] > 0
        table.insert(current_tab, { buf = buf_item[1], tab = buf_item[3][1], matched = matched })
        if calc_width then
            max_width = math.max(max_width, vim.fn.strwidth(buf_item[2]))
        end
    end

    for _, buf_item in ipairs(buffers.other_tabs) do
        local matched = buf_item[4] ~= nil and #buf_item[4] > 0
        table.insert(other_tabs, { buf = buf_item[1], tab = buf_item[3][1], matched = matched })
        if calc_width then
            max_width = math.max(max_width, vim.fn.strwidth(buf_item[2]))
        end
    end

    if calc_width then
        max_width = math.max(max_width, frame_len.titles.max_width)

        local max_total_width = max_width + 2 * frame.padding
        local rem = max_total_width % frame_len.hor.width
        if rem ~= 0 then
            max_width = max_width + (frame_len.hor.width - rem)
        end
    end

    if current_states then
        if calc_width then
            current_states.max_width = max_width
        end

        current_states.items = {
            current_tab = current_tab,
            other_tabs = other_tabs,
        }

        return current_states
    else
        local new_states = {
            max_width = max_width,
            items = {
                current_tab = current_tab,
                other_tabs = other_tabs,
            },
        }
        states.set(new_states)

        return new_states
    end
end

function M.render_results(buffers)
    local current_states = update_states(buffers)

    local lines = {}
    local exts = {}

    lines, exts = render_items(buffers.current_tab, lines, exts, current_states, frame.titles[1])
    lines, exts = render_items(buffers.other_tabs, lines, exts, current_states, frame.titles[2])

    local buf_id = ui.main.get_buf()
    if not buf_id then return end

    ui.main.set_lines(0, -1, false, lines)

    for _, ext in ipairs(exts) do
        for _, ext_item in ipairs(ext) do
            hl.set_extmark[ext_item.hl](buf_id, ext_item)
        end
    end

    local win_id = ui.main.get_win()
    if not win_id then return end
    api.nvim_win_set_cursor(win_id, { 2, 0 })
end

function M.open_results()
    ui.main.create_buf()
    local buf = ui.main.get_buf()
    if not buf then return end
    hl.clear_extmarks(buf)
    ui.main.set_lines(0, -1, false, {})
    ui.main.open_float()

    states.clear()
end

function M.open_input(setup)
    ui.companion.create_buf(setup)
    ui.companion.set_lines(0, -1, false, {})
    vim.schedule(function()
        ui.companion.open_float()
    end)
    vim.cmd("startinsert")
end

function M.get_input()
    local lines = ui.companion.lines(0, 1, false)
    if not lines or #lines == 0 then return "" end
    return lines[1]
end

function M.close()
    if not ui.main.get_win() then return end
    vim.cmd("stopinsert")
    if not myui.focus_on_last_active_ui() then myui.focus_on_last_active_win() end
    ui.main.close()
end

function M.open_selected_buf()
    local win_id = ui.main.get_win()
    if not win_id then return end
    local cursor = api.nvim_win_get_cursor(win_id)
    local line = cursor[1] - 1

    if line <= 0 then return end
    local current_states = states.get()
    if not current_states or line > #current_states.items.current_tab then return end

    local buf_item = current_states.items.current_tab[line]
    if not buf_item.matched then return end
    local buf_id = buf_item.buf

    vim.cmd("stopinsert")
    myui.focus_on_last_active_win()
    myui.close_all()
    api.nvim_set_current_buf(buf_id)
end

M.update_opts = ui.update_opts

function M.autocmd()
    local augroup = api.nvim_create_augroup("NaughieBufferSwitcherUi", { clear = true }),

    api.nvim_create_autocmd("TabLeave", {
        group = augroup,
        callback = M.close,
    })
end

return M
