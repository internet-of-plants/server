CREATE TABLE IF NOT EXISTS organizations (
  id          BIGSERIAL    PRIMARY KEY NOT NULL,
  name        VARCHAR(255)             NOT NULL,
  description TEXT,
  created_at  TIMESTAMPTZ              NOT NULL DEFAULT NOW(),
  updated_at  TIMESTAMPTZ              NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS users (
  id                      BIGSERIAL     PRIMARY KEY NOT NULL,
  email                   VARCHAR(255)              NOT NULL UNIQUE,
  password_hash           VARCHAR(255)              NOT NULL,
  username                VARCHAR(255)              NOT NULL UNIQUE,
  default_organization_id BIGINT                    NOT NULL UNIQUE,
  created_at              TIMESTAMPTZ               NOT NULL DEFAULT NOW(),
  updated_at              TIMESTAMPTZ               NOT NULL DEFAULT NOW(),
  FOREIGN KEY (default_organization_id) REFERENCES  organizations (id),
  CHECK (email <> '' AND username <> '')
);

CREATE TABLE IF NOT EXISTS user_belongs_to_organization (
  id              BIGSERIAL     PRIMARY KEY NOT NULL,
  user_id         BIGINT                    NOT NULL,
  organization_id BIGINT                    NOT NULL,
  created_at      TIMESTAMPTZ               NOT NULL DEFAULT NOW(),
  UNIQUE (user_id, organization_id),
  FOREIGN KEY (user_id)         REFERENCES  users         (id),
  FOREIGN KEY (organization_id) REFERENCES  organizations (id)
);

CREATE TABLE IF NOT EXISTS collections (
  id          BIGSERIAL    PRIMARY KEY NOT NULL,
  name        VARCHAR(255)             NOT NULL,
  description TEXT,
  created_at  TIMESTAMPTZ              NOT NULL DEFAULT NOW(),
  updated_at  TIMESTAMPTZ              NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS collection_belongs_to_organization (
  id              BIGSERIAL     PRIMARY KEY NOT NULL,
  collection_id   BIGINT                    NOT NULL,
  organization_id BIGINT                    NOT NULL,
  created_at      TIMESTAMPTZ               NOT NULL DEFAULT NOW(),
  UNIQUE (collection_id, organization_id),
  FOREIGN KEY (collection_id)   REFERENCES  collections   (id),
  FOREIGN KEY (organization_id) REFERENCES  organizations (id)
);

CREATE TABLE IF NOT EXISTS target_prototypes (
  id                      BIGSERIAL   PRIMARY KEY NOT NULL,
  arch                    TEXT                    NOT NULL,
  build_flags             TEXT                    NOT NULL,
  build_unflags           TEXT,
  platform                TEXT                    NOT NULL,
  framework               TEXT,
  platform_packages       TEXT,
  extra_platformio_params TEXT,
  ldf_mode                TEXT,
  created_at              TIMESTAMPTZ             NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS targets (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  board               TEXT,
  pin_hpp             TEXT                  NOT NULL,
  build_flags         TEXT,
  target_prototype_id BIGINT                NOT NULL,
  FOREIGN KEY (target_prototype_id) REFERENCES target_prototypes (id)
);

CREATE TABLE IF NOT EXISTS compilers (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  target_id  BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (target_id) REFERENCES targets (id)
);

CREATE TABLE IF NOT EXISTS compilations (
  id             BIGSERIAL PRIMARY KEY NOT NULL,
  compiler_id    BIGINT NOT NULL,
  platformio_ini TEXT NOT NULL,
  main_cpp       TEXT NOT NULL,
  pin_hpp        TEXT NOT NULL,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (compiler_id) REFERENCES compilers (id)
);

CREATE TABLE IF NOT EXISTS firmwares (
  id             BIGSERIAL PRIMARY KEY NOT NULL,
  compilation_id BIGINT,
  bin            BYTEA,
  binary_hash    VARCHAR(255) NOT NULL UNIQUE,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (compilation_id) REFERENCES compilations (id)
);

CREATE TABLE IF NOT EXISTS devices (
  id               BIGSERIAL PRIMARY KEY NOT NULL,
  name             TEXT NOT NULL,
  description      TEXT,
  collection_id    BIGINT NOT NULL,
  firmware_id      BIGINT NOT NULL,
  compiler_id      BIGINT,
  mac              CHAR(17) NOT NULL,
  number_of_plants INTEGER NOT NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (compiler_id) REFERENCES compilers (id),
  FOREIGN KEY (collection_id) REFERENCES collections (id),
  FOREIGN KEY (firmware_id) REFERENCES firmwares (id)
);

CREATE TABLE IF NOT EXISTS events (
  id            BIGSERIAL PRIMARY KEY NOT NULL,
  device_id     BIGINT                NOT NULL,
  -- TODO: add type-safety
  measurements  JSONB                 NOT NULL,
  metadatas     JSONB                 NOT NULL,
  firmware_hash VARCHAR(255)          NOT NULL,
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

CREATE TABLE IF NOT EXISTS authentications (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  user_id    BIGINT,
  device_id  BIGINT,
  mac        CHAR(17),
  token      VARCHAR(255) NOT NULL,
  expired    BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (user_id) REFERENCES users (id),
  FOREIGN KEY (device_id) REFERENCES devices (id)
  FOREIGN KEY (mac) REFERENCES devices (mac)
);

CREATE TABLE IF NOT EXISTS sensor_prototypes (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  name       TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sensors (
  id           BIGSERIAL     PRIMARY KEY NOT NULL,
  prototype_id BIGINT                    NOT NULL,
  FOREIGN KEY (prototype_id) REFERENCES  sensor_prototypes (id)
);

CREATE TYPE WidgetKindRaw AS ENUM (
  'U8', 'U16', 'U32', 'U64', 'F32', 'F64', 'String', 'PinSelection', 'Selection'
);

-- TODO: we need some kind of tenancy here
CREATE TABLE IF NOT EXISTS config_types (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  name       TEXT NOT NULL,
  widget     WidgetKindRaw NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS config_type_selection_options (
  id         BIGSERIAL PRIMARY KEY NOT NULL,
  type_id    BIGINT NOT NULL,
  option     TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (type_id) REFERENCES config_types (id)
);

CREATE TABLE IF NOT EXISTS config_requests (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  type_id             BIGINT                NOT NULL,
  name                TEXT                  NOT NULL,
  human_name          TEXT                  NOT NULL,
  sensor_prototype_id BIGINT                NOT NULL,
  created_at          TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  FOREIGN KEY (type_id) REFERENCES config_types (id),
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS configs (
  id                BIGSERIAL PRIMARY KEY NOT NULL,
  sensor_id         BIGINT NOT NULL,
  request_id BIGINT NOT NULL,
  value             TEXT NOT NULL,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (sensor_id) REFERENCES sensors (id),
  FOREIGN KEY (request_id) REFERENCES config_requests (id)
);

CREATE TABLE IF NOT EXISTS pins (
  id       BIGSERIAL PRIMARY KEY NOT NULL,
  target_id BIGINT NOT NULL,
  name     TEXT NOT NULL,
  FOREIGN KEY (target_id) REFERENCES targets (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_includes (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  include             TEXT NOT NULL,
  sensor_prototype_id BIGINT NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_definitions (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  definition          TEXT NOT NULL,
  sensor_prototype_id BIGINT NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_dependencies (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  dependency          TEXT NOT NULL,
  sensor_prototype_id BIGINT NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_setups (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  setup TEXT NOT NULL,
  sensor_prototype_id BIGINT NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TYPE MeasurementType AS ENUM (
  'FloatCelsius', 'Percentage', 'RawAnalogRead'
);

CREATE TYPE MeasurementKind AS ENUM (
  'SoilTemperature', 'SoilMoisture', 'AirTemperature', 'AirHumidity'
);

CREATE TABLE IF NOT EXISTS sensor_prototype_measurements (
  id                  BIGSERIAL PRIMARY KEY NOT NULL,
  name                TEXT                  NOT NULL,
  human_name          TEXT                  NOT NULL,
  value               TEXT                  NOT NULL,
  sensor_prototype_id BIGINT                NOT NULL,
  ty                  MeasurementType       NOT NULL,
  kind                MeasurementKind       NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_belongs_to_compiler (
  id          BIGSERIAL PRIMARY KEY NOT NULL,
  compiler_id BIGINT                NOT NULL,
  sensor_id   BIGINT                NOT NULL,
  device_id     BIGINT                NOT NULL,
  alias       TEXT                  NOT NULL,
  created_at  TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
  UNIQUE (compiler_id, sensor_id),
  FOREIGN KEY (device_id) REFERENCES devices (id),
  FOREIGN KEY (compiler_id) REFERENCES compilers (id),
  FOREIGN KEY (sensor_id) REFERENCES sensors (id)
);
