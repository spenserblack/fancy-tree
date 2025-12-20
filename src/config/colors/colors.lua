return {
  ---@param filepath string
  ---@param attributes FileAttributes
  ---@param default Color|nil
  ---@return Color|nil
  icons = function(filepath, attributes, default)
    return default
  end,
  git_statuses = {
    ---@param status GitStatus
    ---@param default Color|nil
    ---@return Color|nil
    untracked = function(status, default)
      return default
    end,
    ---@param status GitStatus
    ---@param default Color|nil
    ---@return Color|nil
    tracked = function(status, default)
      return default
    end,
  },
}
