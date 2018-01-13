CREATE OR REPLACE FUNCTION converterino(
  x BIGINT
)
RETURNS VARCHAR(20) AS $res$
DECLARE
  res VARCHAR(20);
BEGIN
  RETURN CASE
      WHEN x >= 0 THEN CAST(x AS VARCHAR(20))
      ELSE CAST(x + POW(2, CAST(64 AS NUMERIC(20, 0))) AS VARCHAR(20))
  END;
END;
$res$ LANGUAGE plpgsql;

ALTER TABLE administrators
  ALTER COLUMN user_id TYPE VARCHAR(20) USING converterino(CAST(user_id AS NUMERIC));

ALTER TABLE auto_replies
  ALTER COLUMN server_id TYPE VARCHAR(20) USING converterino(CAST(server_id AS NUMERIC)),
  ALTER COLUMN channel_id TYPE VARCHAR(20) USING converterino(CAST(channel_id AS NUMERIC));

ALTER TABLE channel_configs
  ALTER COLUMN server_id TYPE VARCHAR(20) USING converterino(CAST(server_id AS NUMERIC)),
  ALTER COLUMN channel_id TYPE VARCHAR(20) USING converterino(CAST(channel_id AS NUMERIC));

ALTER TABLE delete_all_messages
  ALTER COLUMN server_id TYPE VARCHAR(20) USING converterino(CAST(server_id AS NUMERIC)),
  ALTER COLUMN channel_id TYPE VARCHAR(20) USING converterino(CAST(channel_id AS NUMERIC));

ALTER TABLE reactions
  ALTER COLUMN server_id TYPE VARCHAR(20) USING converterino(CAST(server_id AS NUMERIC)),
  ALTER COLUMN channel_id TYPE VARCHAR(20) USING converterino(CAST(channel_id AS NUMERIC)),
  ALTER COLUMN message_id TYPE VARCHAR(20) USING converterino(CAST(message_id AS NUMERIC));

ALTER TABLE role_check_times
  ALTER COLUMN user_id TYPE VARCHAR(20) USING converterino(CAST(user_id AS NUMERIC));

ALTER TABLE server_configs
  ALTER COLUMN server_id TYPE VARCHAR(20) USING converterino(CAST(server_id AS NUMERIC));

ALTER TABLE tags
  ALTER COLUMN user_id TYPE VARCHAR(20) USING converterino(CAST(user_id AS NUMERIC)),
  ALTER COLUMN server_id TYPE VARCHAR(20) USING converterino(CAST(server_id AS NUMERIC)),
  ALTER COLUMN character_id TYPE VARCHAR(20) USING converterino(CAST(character_id AS NUMERIC));

ALTER TABLE timeouts
  ALTER COLUMN user_id TYPE VARCHAR(20) USING converterino(CAST(user_id AS NUMERIC)),
  ALTER COLUMN server_id TYPE VARCHAR(20) USING converterino(CAST(server_id AS NUMERIC)),
  ALTER COLUMN role_id TYPE VARCHAR(20) USING converterino(CAST(role_id AS NUMERIC));
