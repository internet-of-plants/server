CREATE TABLE IF NOT EXISTS sensor_prototype_authenticated_actions (
  id                     BIGSERIAL   PRIMARY KEY NOT NULL,
  authenticated_action   TEXT        NOT NULL,
  sensor_prototype_id    BIGINT      NOT NULL,
  UNIQUE (authenticated_action, sensor_prototype_id),
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);
