local M = {}

local default_ns = "buffer-switcher"

local router = require("nvim-router")

local rpc = { request = function() end, notify = function() end }

function M.register(plugin_dir, new_ns)
    local info = {
        path = plugin_dir .. "/buffer-switcher.rs",
        handler = "NeovimHandler",
    }

    if new_ns then
        info.ns = new_ns
    else
        info.ns = default_ns
    end

    local new_rpc = router.register(info)
    rpc.request = new_rpc.request
    rpc.notify = new_rpc.notify
end

M.call = {
    update_buffers = function(buffers)
        local cwd = vim.uv.cwd()
        rpc.notify("update_buffers", buffers.current_tab, buffers.other_tabs, cwd)
    end,

    rerank = function(input)
        return rpc.request("rank", input)
    end,
}

return M
