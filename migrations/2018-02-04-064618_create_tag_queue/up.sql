CREATE TABLE tag_queue (
  id SERIAL PRIMARY KEY,
  server_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  server TEXT NOT NULL,
  character TEXT NOT NULL
)
