CREATE TABLE timeouts (
  id SERIAL PRIMARY KEY,
  user_id TEXT NOT NULL,
  server_id TEXT NOT NULL,
  role_id TEXT NOT NULL,
  seconds INTEGER NOT NULL,
  start BIGINT NOT NULL
)
