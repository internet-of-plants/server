CREATE TABLE IF NOT EXISTS organizations (
  id          BIGSERIAL    PRIMARY KEY NOT NULL,
  name        TEXT                     NOT NULL UNIQUE,
  description TEXT,
  created_at  TIMESTAMPTZ              NOT NULL DEFAULT NOW(),
  updated_at  TIMESTAMPTZ              NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS users (
  id                      BIGSERIAL     PRIMARY KEY NOT NULL,
  email                   TEXT                      NOT NULL UNIQUE,
  password_hash           TEXT                      NOT NULL,
  username                TEXT                      NOT NULL UNIQUE,
  default_organization_id BIGINT                    NOT NULL UNIQUE,
  created_at              TIMESTAMPTZ               NOT NULL DEFAULT NOW(),
  updated_at              TIMESTAMPTZ               NOT NULL DEFAULT NOW(),
  FOREIGN KEY (default_organization_id) REFERENCES  organizations (id),
  CHECK (email <> '' AND username <> '')
);

CREATE TABLE IF NOT EXISTS user_belongs_to_organization (
  user_id         BIGINT                    NOT NULL,
  organization_id BIGINT                    NOT NULL,
  created_at      TIMESTAMPTZ               NOT NULL DEFAULT NOW(),
  UNIQUE (user_id, organization_id),
  FOREIGN KEY (user_id)         REFERENCES  users         (id),
  FOREIGN KEY (organization_id) REFERENCES  organizations (id)
);

CREATE TABLE IF NOT EXISTS target_prototypes (
  id                      BIGSERIAL   PRIMARY KEY NOT NULL,
  certs_url               TEXT                    NOT NULL,
  arch                    TEXT                    NOT NULL UNIQUE,
  build_flags             TEXT                    NOT NULL,
  build_unflags           TEXT,
  platform                TEXT                    NOT NULL,
  framework               TEXT,
  platform_packages       TEXT,
  extra_platformio_params TEXT,
  ldf_mode                TEXT,
  created_at              TIMESTAMPTZ             NOT NULL DEFAULT NOW(),
  UNIQUE (arch)
);

CREATE TABLE IF NOT EXISTS dependency_belongs_to_target_prototype (
  target_prototype_id BIGINT                    NOT NULL,
  repo_url            TEXT                      NOT NULL,
  branch              TEXT                      NOT NULL,
  created_at          TIMESTAMPTZ               NOT NULL DEFAULT NOW(),
  UNIQUE (target_prototype_id, repo_url),
  FOREIGN KEY (target_prototype_id) REFERENCES target_prototypes (id)
);

CREATE TABLE IF NOT EXISTS targets (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  board               TEXT,
  pin_hpp             TEXT                  NOT NULL,
  build_flags         TEXT,
  target_prototype_id BIGINT                NOT NULL,
  FOREIGN KEY (target_prototype_id) REFERENCES target_prototypes (id)
);

CREATE UNIQUE INDEX board_target ON targets (board, target_prototype_id) WHERE board IS NOT NULL;
CREATE UNIQUE INDEX board_target_null ON targets (target_prototype_id) WHERE board IS NULL;


CREATE TABLE IF NOT EXISTS compilers (
  id              BIGSERIAL PRIMARY KEY NOT NULL,
  organization_id BIGINT                NOT NULL,
  target_id       BIGINT                NOT NULL,
  created_at      TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  FOREIGN KEY (organization_id) REFERENCES organizations (id),
  FOREIGN KEY (target_id) REFERENCES targets (id)
);

CREATE TABLE IF NOT EXISTS collections (
  id                  BIGSERIAL    PRIMARY KEY NOT NULL,
  name                TEXT                     NOT NULL,
  compiler_id         BIGINT,
  description         TEXT,
  target_prototype_id BIGINT                   NOT NULL,
  created_at          TIMESTAMPTZ              NOT NULL DEFAULT NOW(),
  updated_at          TIMESTAMPTZ              NOT NULL DEFAULT NOW(),
  FOREIGN KEY (compiler_id) REFERENCES compilers (id)
);

CREATE TABLE IF NOT EXISTS collection_belongs_to_organization (
  collection_id   BIGINT                    NOT NULL,
  organization_id BIGINT                    NOT NULL,
  created_at      TIMESTAMPTZ               NOT NULL DEFAULT NOW(),
  UNIQUE (collection_id, organization_id),
  FOREIGN KEY (collection_id)   REFERENCES  collections   (id),
  FOREIGN KEY (organization_id) REFERENCES  organizations (id)
);

CREATE TABLE IF NOT EXISTS certificates (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  target_prototype_id BIGINT                NOT NULL,
  hash                TEXT                  NOT NULL,
  payload             BYTEA                 NOT NULL,
  created_at          TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  UNIQUE (target_prototype_id, hash),
  FOREIGN KEY (target_prototype_id) REFERENCES target_prototypes (id)
);

CREATE TABLE IF NOT EXISTS compilations (
  id             BIGSERIAL PRIMARY KEY NOT NULL,
  compiler_id    BIGINT                NOT NULL,
  platformio_ini TEXT                  NOT NULL,
  main_cpp       TEXT                  NOT NULL,
  pin_hpp        TEXT                  NOT NULL,
  certificate_id BIGINT                NOT NULL,
  created_at     TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  FOREIGN KEY (compiler_id) REFERENCES compilers (id),
  FOREIGN KEY (certificate_id) REFERENCES certificates (id)
);

-- TODO: connect to target prototype id and maybe target_id
CREATE TABLE IF NOT EXISTS firmwares (
  id              BIGSERIAL PRIMARY KEY NOT NULL,
  compilation_id  BIGINT,
  bin             BYTEA,
  organization_id BIGINT                NOT NULL,
  binary_hash     TEXT                  NOT NULL,
  created_at      TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  UNIQUE (organization_id, binary_hash),
  FOREIGN KEY (organization_id) REFERENCES organizations (id),
  FOREIGN KEY (compilation_id) REFERENCES compilations (id)
);

CREATE TABLE IF NOT EXISTS devices (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  name                TEXT                  NOT NULL,
  description         TEXT,
  collection_id       BIGINT                NOT NULL,
  firmware_id         BIGINT                NOT NULL,
  mac                 CHAR(17)              NOT NULL,
  target_prototype_id BIGINT                NOT NULL,
  created_at          TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  updated_at          TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  UNIQUE (mac, collection_id),
  FOREIGN KEY (collection_id) REFERENCES collections (id),
  FOREIGN KEY (target_prototype_id) REFERENCES target_prototypes (id),
  FOREIGN KEY (firmware_id) REFERENCES firmwares (id)
);

CREATE TABLE IF NOT EXISTS events (
  id            BIGSERIAL PRIMARY KEY NOT NULL,
  device_id     BIGINT                NOT NULL,
  measurements  JSONB                 NOT NULL,
  stat          JSONB                 NOT NULL,
  metadatas     JSONB                 NOT NULL,
  firmware_hash TEXT                  NOT NULL,
  created_at    TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  FOREIGN KEY (device_id) REFERENCES devices (id)
);

CREATE TABLE IF NOT EXISTS device_panics (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  device_id  BIGINT NOT NULL,
  file       TEXT NOT NULL,
  line       INT NOT NULL,
  func       TEXT NOT NULL,
  msg        TEXT NOT NULL,
  is_solved  BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (device_id) REFERENCES devices (id)
);

CREATE TABLE IF NOT EXISTS device_logs (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  device_id  BIGINT NOT NULL,
  log        TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (device_id) REFERENCES devices (id)
);

-- TODO: forbid multiple active tokens for the same device
CREATE TABLE IF NOT EXISTS authentications (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  user_id    BIGINT,
  device_id  BIGINT,
  mac        CHAR(17),
  token      TEXT                  NOT NULL,
  expired    BOOLEAN               NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  FOREIGN KEY (user_id) REFERENCES users (id),
  FOREIGN KEY (device_id) REFERENCES devices (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototypes (
  id            BIGSERIAL PRIMARY KEY NOT NULL,
  name          TEXT NOT NULL UNIQUE,
  variable_name TEXT,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- TODO: add organization filter here and in a bunch of other tables
CREATE TABLE IF NOT EXISTS sensors (
  id           BIGSERIAL     PRIMARY KEY NOT NULL,
  prototype_id BIGINT                    NOT NULL,
  index        BIGINT                    NOT NULL,
  FOREIGN KEY (prototype_id) REFERENCES  sensor_prototypes (id)
);

CREATE TYPE SensorWidgetKindRaw AS ENUM (
  'Seconds', 'U8', 'U16', 'U32', 'U64', 'F32', 'F64', 'String', 'PinSelection', 'Selection', 'Moment', 'Map', 'Sensor'
);

CREATE TABLE IF NOT EXISTS sensor_config_types (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  name       TEXT,
  widget     SensorWidgetKindRaw NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TYPE ParentMetadata AS ENUM (
  'Key', 'Value'
);

CREATE TABLE IF NOT EXISTS sensor_config_type_selection_maps (
  id              BIGSERIAL           PRIMARY KEY NOT NULL,
  type_id         BIGINT              NOT NULL,
  parent_id       BIGINT,
  parent_metadata ParentMetadata,
  key             SensorWidgetKindRaw NOT NULL,
  value           SensorWidgetKindRaw NOT NULL,
  created_at      TIMESTAMPTZ         NOT NULL DEFAULT NOW(),
  UNIQUE (type_id, parent_id, parent_metadata, key),
  FOREIGN KEY (type_id) REFERENCES sensor_config_types (id),
  FOREIGN KEY (parent_id) REFERENCES sensor_config_type_selection_maps (id)
);

CREATE TABLE IF NOT EXISTS sensor_config_type_selection_options (
  id                  BIGSERIAL       PRIMARY KEY NOT NULL,
  type_id             BIGINT          NOT NULL,
  parent_id           BIGINT,
  parent_metadata     ParentMetadata,
  option              TEXT            NOT NULL,
  created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
  UNIQUE (type_id, parent_id, parent_metadata, option),
  FOREIGN KEY (type_id) REFERENCES sensor_config_types (id),
  FOREIGN KEY (parent_id) REFERENCES sensor_config_type_selection_maps (id)
);

CREATE TABLE IF NOT EXISTS sensor_config_type_selection_sensors (
  id                    BIGSERIAL       PRIMARY KEY NOT NULL,
  type_id               BIGINT          NOT NULL,
  parent_id             BIGINT,
  parent_metadata       ParentMetadata,
  sensor_prototype_name TEXT            NOT NULL,
  created_at            TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
  UNIQUE (type_id, parent_id, parent_metadata, sensor_prototype_name),
  FOREIGN KEY (type_id) REFERENCES sensor_config_types (id),
  FOREIGN KEY (parent_id) REFERENCES sensor_config_type_selection_maps (id)
);

CREATE TABLE IF NOT EXISTS sensor_config_requests (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  type_id             BIGINT                NOT NULL,
  name                TEXT                  NOT NULL,
  variable_name       TEXT                  NOT NULL,
  sensor_prototype_id BIGINT                NOT NULL,
  created_at          TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  UNIQUE (variable_name, sensor_prototype_id),
  FOREIGN KEY (type_id) REFERENCES sensor_config_types (id),
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_configs (
  id                BIGSERIAL PRIMARY KEY NOT NULL,
  sensor_id         BIGINT                NOT NULL,
  request_id        BIGINT                NOT NULL,
  value             JSONB                 NOT NULL,
  created_at        TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  FOREIGN KEY (sensor_id) REFERENCES sensors (id),
  FOREIGN KEY (request_id) REFERENCES sensor_config_requests (id)
);

CREATE TYPE DeviceWidgetKind AS ENUM (
  'SSID', 'PSK', 'Timezone'
);

CREATE TABLE IF NOT EXISTS device_config_types (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  name       TEXT NOT NULL,
  widget     DeviceWidgetKind NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (name, widget)
);

CREATE TABLE IF NOT EXISTS device_config_type_selection_options (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  type_id    BIGINT NOT NULL,
  option     TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (type_id, option),
  FOREIGN KEY (type_id) REFERENCES device_config_types (id)
);

CREATE TABLE IF NOT EXISTS device_config_requests (
  id            BIGSERIAL PRIMARY KEY NOT NULL,
  type_id       BIGINT                NOT NULL,
  name          TEXT                  NOT NULL,
  variable_name TEXT                  NOT NULL,
  target_id     BIGINT                NOT NULL,
  created_at    TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  UNIQUE (type_id, variable_name, target_id),
  FOREIGN KEY (type_id) REFERENCES device_config_types (id),
  FOREIGN KEY (target_id) REFERENCES targets (id)
);

CREATE TABLE IF NOT EXISTS device_configs (
  id                BIGSERIAL PRIMARY KEY NOT NULL,
  organization_id   BIGINT                NOT NULL,
  request_id        BIGINT                NOT NULL,
  value             TEXT                  NOT NULL,
  created_at        TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  UNIQUE (organization_id, request_id, value),
  FOREIGN KEY (organization_id) REFERENCES organizations (id),
  FOREIGN KEY (request_id) REFERENCES device_config_requests (id)
);
CREATE TABLE IF NOT EXISTS device_config_belongs_to_compiler (
  compiler_id BIGINT                NOT NULL,
  config_id   BIGINT                NOT NULL,
  created_at  TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  UNIQUE (compiler_id, config_id),
  FOREIGN KEY (compiler_id) REFERENCES compilers (id),
  FOREIGN KEY (config_id) REFERENCES device_configs (id)
);

CREATE TABLE IF NOT EXISTS pins (
  id        BIGSERIAL PRIMARY KEY NOT NULL,
  target_id BIGINT                NOT NULL,
  name      TEXT                   NOT NULL,
  UNIQUE (target_id, name),
  FOREIGN KEY (target_id) REFERENCES targets (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_includes (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  include             TEXT                  NOT NULL,
  sensor_prototype_id BIGINT                NOT NULL,
  UNIQUE (include, sensor_prototype_id),
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_definitions (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  line                TEXT                  NOT NULL,
  sensor_prototype_id BIGINT                NOT NULL,
  UNIQUE (line, sensor_prototype_id),
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_definition_sensors_referenced (
  id                             BIGSERIAL PRIMARY KEY NOT NULL,
  sensor_name                    TEXT                  NOT NULL,
  request_name                   TEXT                  NOT NULL,
  sensor_prototype_definition_id BIGINT                NOT NULL,
  UNIQUE (request_name, sensor_prototype_definition_id),
  FOREIGN KEY (sensor_prototype_definition_id) REFERENCES sensor_prototype_definitions (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_dependencies (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  repo_url            TEXT                  NOT NULL,
  branch              TEXT                  NOT NULL,
  sensor_prototype_id BIGINT                NOT NULL,
  UNIQUE (repo_url, sensor_prototype_id),
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_setups (
  id                  BIGSERIAL   PRIMARY KEY NOT NULL,
  setup               TEXT        NOT NULL,
  sensor_prototype_id BIGINT      NOT NULL,
  UNIQUE (setup, sensor_prototype_id),
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_unauthenticated_actions (
  id                     BIGSERIAL   PRIMARY KEY NOT NULL,
  unauthenticated_action TEXT        NOT NULL,
  sensor_prototype_id    BIGINT      NOT NULL,
  UNIQUE (unauthenticated_action, sensor_prototype_id),
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TYPE SensorMeasurementType AS ENUM (
  'FloatCelsius', 'Percentage', 'RawAnalogRead'
);

CREATE TYPE SensorMeasurementKind AS ENUM (
  'SoilTemperature', 'SoilMoisture', 'AirTemperature', 'AirHumidity'
);

CREATE TABLE IF NOT EXISTS sensor_prototype_measurements (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  name                TEXT                  NOT NULL,
  variable_name       TEXT                  NOT NULL,
  value               TEXT                  NOT NULL,
  sensor_prototype_id BIGINT                NOT NULL,
  ty                  SensorMeasurementType       NOT NULL,
  kind                SensorMeasurementKind       NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_belongs_to_compiler (
  compiler_id BIGINT                NOT NULL,
  sensor_id   BIGINT                NOT NULL,
  alias       TEXT                  NOT NULL,
  color       TEXT                  NOT NULl,
  created_at  TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  updated_at  TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  UNIQUE (compiler_id, sensor_id),
  FOREIGN KEY (compiler_id) REFERENCES compilers (id),
  FOREIGN KEY (sensor_id) REFERENCES sensors (id)
);

CREATE TABLE IF NOT EXISTS dependency_belongs_to_compilation (
  repo_url        TEXT                      NOT NULL,
  branch          TEXT                      NOT NULL,
  sensor_id       BIGINT,
  commit_hash     TEXT                      NOT NULL,
  compilation_id  BIGINT                    NOT NULL,
  created_at      TIMESTAMPTZ               NOT NULL DEFAULT NOW(),
  UNIQUE (repo_url, compilation_id),
  FOREIGN KEY (sensor_id)      REFERENCES sensors     (id),
  FOREIGN KEY (compilation_id) REFERENCES compilations (id)
);

