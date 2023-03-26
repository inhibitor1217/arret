local function fixedWindow(
  slot,
  capacity,
  window,
  requestedTokens
)
  -- Retrieve the current bucket for the key,
  -- or create a new one if it doesn't exist
  local bucket = redis.call("GET", slot)
  if bucket == false then
    bucket = capacity
  else
    bucket = tonumber(bucket)
  end

  if bucket < requestedTokens then
    -- Not enough tokens
    return {false, bucket}
  else
    -- Consume the tokens in the current window
    -- Expiration should be set so that past slots do not take up space
    bucket = bucket - requestedTokens

    redis.call("SET", slot, bucket, "EX", window)

    return {true, bucket}
  end
end

return fixedWindow(
  KEYS[1],
  tonumber(ARGV[1]),
  tonumber(ARGV[2]),
  tonumber(ARGV[3])
)
