CREATE TABLE auto_replies (
  id SERIAL PRIMARY KEY,
  server_id TEXT NOT NULL,
  channel_id TEXT NOT NULL,
  message TEXT NOT NULL,
  on_join BOOLEAN NOT NULL,
  delay INTEGER NOT NULL,
  filters TEXT
)
