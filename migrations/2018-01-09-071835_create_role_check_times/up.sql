CREATE TABLE role_check_times (
  id SERIAL PRIMARY KEY,
  check_id INTEGER NOT NULL,
  user_id TEXT NOT NULL,
  reminded_at BIGINT NOT NULL,
  kick_after INTEGER NOT NULL
)
