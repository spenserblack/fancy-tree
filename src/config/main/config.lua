return {
  ---@type "auto"|"on"|"ansi"|"off"|nil
  color = "auto",
  ---@param filepath string Path to the file relative to the starting directory
  ---@param attributes FileAttributes
  ---@param default boolean
  ---@return boolean
  skip = function(filepath, attributes, default)
    -- The default is to hide dotfiles on Unix and files with the hidden attribute on
    -- Windows.
    return default
  end,
  ---@type Sorting|nil
  -- When this is nil, the default sorting algorithm will be used.
  sorting = nil,
  ---@type integer|nil
  -- When this is not nil, it will set how many levels deep this tool should search in
  -- the directory tree.
  level = nil,
}
