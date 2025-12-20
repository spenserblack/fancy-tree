---@meta

---@class FileAttributes
---@field file_type "directory"|"file"|"symlink"
---@field is_hidden boolean
---@field is_executable boolean
---@field language string|nil

---@class RGB
---@field r integer
---@field g integer
---@field b integer

---@alias ANSI "black"|"red"|"green"|"yellow"|"blue"|"magenta"|"cyan"|"white"|"bright-black"|"bright-red"|"bright-green"|"bright-yellow"|"bright-blue"|"bright-magenta"|"bright-cyan"|"bright-white"
---@alias Color ANSI|RGB

---@alias GitStatus "added"|"modified"|"removed"|"renamed"
