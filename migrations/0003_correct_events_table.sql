DROP TABLE event;
CREATE TABLE events (
  id SERIAL PRIMARY KEY,
  _type VARCHAR(16),
  data JSON
);
