local function tokenBucket(
  key,
  now,
  capacity,
  refillInterval,
  refillAmount,
  requestedTokens
)
  -- Retrieve the current bucket for the key,
  -- or create a new one if it doesn't exist
  local bucket = redis.call("GET", key)
  if bucket == false then
    bucket = {capacity, now}
  else
    bucket = cjson.decode(bucket)
  end

  local tokens = bucket[1]
  local lastUpdatedAt = bucket[2]

  -- Refill the token bucket
  local intervalsPassed = math.floor((now - lastUpdatedAt) / refillInterval)
  tokens = math.min(capacity, tokens + (intervalsPassed * refillAmount))
  lastUpdatedAt = lastUpdatedAt + (intervalsPassed * refillInterval)

  if tokens < requestedTokens then
    -- Not enough tokens
    return {false, tokens, lastUpdatedAt + refillInterval}
  else
    -- Consume the tokens, and update the token bucket
    -- Expiration should be set so that full buckets do not take up space
    tokens = tokens - requestedTokens
    local ttl = refillInterval * math.ceil((capacity - tokens) / refillAmount)

    redis.call("SET", key, cjson.encode({tokens, lastUpdatedAt}), "EX", ttl)

    return {true, tokens, lastUpdatedAt + refillInterval}
  end
end

return tokenBucket(
  KEYS[1],
  tonumber(ARGV[1]),
  tonumber(ARGV[2]),
  tonumber(ARGV[3]),
  tonumber(ARGV[4]),
  tonumber(ARGV[5])
)
