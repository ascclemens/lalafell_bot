CREATE TABLE server_configs (
  id SERIAL PRIMARY KEY,
  server_id TEXT NOT NULL,
  timeout_role TEXT
)
