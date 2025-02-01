CREATE TABLE user_register_events (
  id SERIAL PRIMARY KEY,
  key VARCHAR NOT NULL,
  user_login VARCHAR NOT NULL,
  password_hash VARCHAR NOT NULL,
  details JSON NOT NULL
);

CREATE TABLE user_login_events (
  id SERIAL PRIMARY KEY,
  key VARCHAR NOT NULL,
  user_login VARCHAR NOT NULL
);

CREATE TABLE user_delete_events (
  id SERIAL PRIMARY KEY,
  key VARCHAR NOT NULL,
  user_login VARCHAR NOT NULL
);

DROP TABLE events;
