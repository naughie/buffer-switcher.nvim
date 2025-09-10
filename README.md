# Buffer-switcher

Buffer-switcher is a Neovim plugin for the tab-wise version of `:buffer {bufname}`.


# Requirements

- Rust (>= 1.88.0)


# Install

After `nvim-router` detects that all of dependencies, which are specified in `opts.ns` of `nvim-router` itself, are `setup`'d, then it automatically runs `cargo build --release` and spawns a plugin-client process.

The first build may take a long time.

## Lazy.nvim

### Config

```lua
{
    -- Dependencies
    { "naughie/glocal-states.nvim", lazy = true },
    { "naughie/my-ui.nvim", lazy = true },

    {
        "naughie/nvim-router.nvim",
        lazy = false,
        opts = function(plugin)
            return {
                plugin_dir = plugin.dir,
                ns = { "buffer-switcher" },
            }
        end,
    },

    {
        "naughie/buffer-switcher.nvim",
        lazy = false,
        opts = function(plugin)
            return {
                plugin_dir = plugin.dir,
                rpc_ns = "buffer-switcher",

                border = {
                    -- Highlight group for the border of floating windows.
                    -- Defaults to FloatBorder
                    hl_group = "FloatBorder",
                },
                -- Override highlight groups.
                -- You see all of the available highlights and their default values in the ./lua/buffer-switcher/highlight.lua.
                hl = {
                    matched = { link = "IncSearch" },
                },

                -- { {mode}, {lhs}, {rhs}, {opts} } (see :h vim.keymap.set())
                -- We accept keys of require('buffer-switcher').fn as {rhs}.
                keymaps = {
                    global = {
                        -- Open the buffer list and enter the insert mode.
                        { 'n', '<Space>b', 'open' },
                    },

                    -- Keymaps on an input window.
                    input = {
                        -- Close the windows.
                        { 'i', '<ESC>', 'close' },

                        -- Open the selected buffer. By default, the highest matched buffer is selected.
                        { 'i', '<CR>', 'open_selected_buf' },
                    },
                },
            }
        end,
    },
}
```

