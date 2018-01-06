CREATE TABLE verifications (
  id SERIAL PRIMARY KEY,
  tag_id INTEGER NOT NULL REFERENCES tags(id),
  verified BOOLEAN NOT NULL DEFAULT 'f',
  verification_string VARCHAR
)
