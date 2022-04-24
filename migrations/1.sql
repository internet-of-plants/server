CREATE TABLE IF NOT EXISTS organizations (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS users (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  email VARCHAR(255) NOT NULL UNIQUE,
  password_hash VARCHAR(255) NOT NULL,
  username VARCHAR(255) NOT NULL UNIQUE,
  default_organization_id BIGINT NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (default_organization_id) REFERENCES organizations (id),
  CHECK (email <> '' AND username <> '')
);

CREATE TABLE IF NOT EXISTS user_belongs_to_organization (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  user_id BIGINT NOT NULL,
  organization_id BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (user_id) REFERENCES users (id),
  FOREIGN KEY (organization_id) REFERENCES organizations (id)
);

CREATE TABLE IF NOT EXISTS collections (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS collection_belongs_to_organization (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  collection_id BIGINT NOT NULL,
  organization_id BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (collection_id) REFERENCES collections (id),
  FOREIGN KEY (organization_id) REFERENCES organizations (id)
);

CREATE TABLE IF NOT EXISTS devices (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  collection_id BIGINT NOT NULL,
  name TEXT,
  description TEXT,
  mac CHAR(17) NOT NULL,
  file_hash CHAR(255) NOT NULL,
  number_of_plants INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (collection_id) REFERENCES collections (id)
);

CREATE TABLE IF NOT EXISTS events (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  device_id BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (device_id) REFERENCES devices (id)
);

-- TODO: support dynamic jsonb measurements with custom parser/analyzer
CREATE TABLE IF NOT EXISTS measurements (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  air_temperature_celsius DOUBLE PRECISION NOT NULL,
  air_humidity_percentage DOUBLE PRECISION NOT NULL,
  air_heat_index_celsius DOUBLE PRECISION NOT NULL,
  soil_temperature_celsius DOUBLE PRECISION NOT NULL,
  soil_resistivity_raw SMALLINT NOT NULL,
  event_id BIGINT NOT NULL,
  firmware_hash VARCHAR(255) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (event_id) REFERENCES events (id)
);

CREATE TABLE IF NOT EXISTS device_panics (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  device_id BIGINT NOT NULL,
  file TEXT NOT NULL,
  line INT NOT NULL,
  func TEXT NOT NULL,
  msg TEXT NOT NULL,
  is_solved BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (device_id) REFERENCES devices (id)
);

CREATE TABLE IF NOT EXISTS device_logs (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  device_id BIGINT NOT NULL,
  log TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (device_id) REFERENCES devices (id)
);

CREATE TABLE IF NOT EXISTS authentications (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  user_id BIGINT NOT NULL,
  device_id BIGINT,
  token VARCHAR(255) NOT NULL,
  expired BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (user_id) REFERENCES users (id),
  FOREIGN KEY (device_id) REFERENCES devices (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototypes (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sensors (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  owner_id BIGINT NOT NULL,
  prototype_id BIGINT NOT NULL,
  FOREIGN KEY (owner_id) REFERENCES users (id),
  FOREIGN KEY (prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TYPE WidgetKindRaw AS ENUM (
  'U8', 'U16', 'U32', 'U64', 'F32', 'F64', 'String', 'PinSelection', 'Selection'
);

-- TODO: we need some kind of tenancy here
CREATE TABLE IF NOT EXISTS config_types (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  widget WidgetKindRaw NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS config_type_selection_options (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  type_id BIGINT NOT NULL,
  option TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (type_id) REFERENCES config_types (id)
);

CREATE TABLE IF NOT EXISTS config_requests (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  type_id BIGINT NOT NULL,
  name TEXT NOT NULL,
  sensor_prototype_id BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (type_id) REFERENCES config_types (id),
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS configs (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  sensor_id BIGINT NOT NULL,
  -- target_id BIGINT, -- TODO: fix this, PinSelection config_requests require targets
  config_request_id BIGINT NOT NULL,
  owner_id BIGINT NOT NULL,
  value TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (sensor_id) REFERENCES sensors (id),
  FOREIGN KEY (owner_id) REFERENCES users (id),
  --FOREIGN KEY (target_id) REFERENCES targets (id),
  FOREIGN KEY (config_request_id) REFERENCES config_requests (id)
);

CREATE TABLE IF NOT EXISTS target_prototypes (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  arch TEXT NOT NULL,
  build_flags TEXT NOT NULL,
  platform TEXT NOT NULL,
  framework TEXT,
  platform_packages TEXT NOT NULL,
  extra_platformio_params TEXT NOT NULL,
  ldf_mode TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS boards (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  board TEXT NOT NULL,
  pin_hpp TEXT NOT NULL,
  target_prototype_id BIGINT NOT NULL,
  FOREIGN KEY (target_prototype_id) REFERENCES target_prototypes (id)
);

CREATE TABLE IF NOT EXISTS board_pins (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  board_id BIGINT NOT NULL,
  name TEXT NOT NULL,
  FOREIGN KEY (board_id) REFERENCES boards (id)
);

CREATE TABLE IF NOT EXISTS targets (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  owner_id BIGINT NOT NULL,
  board_id BIGINT NOT NULL,
  target_prototype_id BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (target_prototype_id) REFERENCES target_prototypes (id),
  FOREIGN KEY (board_id) REFERENCES boards (id),
  FOREIGN KEY (owner_id) REFERENCES users (id),
  UNIQUE (owner_id, board_id, target_prototype_id) -- owner should actually be collection
);

CREATE TABLE IF NOT EXISTS sensor_prototype_includes (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  include TEXT NOT NULL,
  sensor_prototype_id BIGINT NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_definitions (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  definition TEXT NOT NULL,
  sensor_prototype_id BIGINT NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_dependencies (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  dependency TEXT NOT NULL,
  sensor_prototype_id BIGINT NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_setups (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  setup TEXT NOT NULL,
  sensor_prototype_id BIGINT NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS sensor_prototype_measurements (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  value TEXT NOT NULL,
  sensor_prototype_id BIGINT NOT NULL,
  FOREIGN KEY (sensor_prototype_id) REFERENCES sensor_prototypes (id)
);

CREATE TABLE IF NOT EXISTS compilers (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  target_id BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (target_id) REFERENCES targets (id)
);

CREATE TABLE IF NOT EXISTS sensor_belongs_to_compiler (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  compiler_id BIGINT NOT NULL,
  sensor_id BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (compiler_id) REFERENCES compilers (id),
  FOREIGN KEY (sensor_id) REFERENCES sensors (id)
);

CREATE TABLE IF NOT EXISTS compilations (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  compiler_id BIGINT NOT NULL,
  platformio_ini TEXT NOT NULL,
  main_cpp TEXT NOT NULL,
  pin_hpp TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (compiler_id) REFERENCES compilers (id)
);

CREATE TABLE IF NOT EXISTS firmwares (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  compilation_id BIGINT,
  bin BYTEA NOT NULL,
  binary_hash VARCHAR(255) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (compilation_id) REFERENCES compilations (id)
);

CREATE TABLE IF NOT EXISTS binary_updates (
  id BIGSERIAL PRIMARY KEY NOT NULL,
  collection_id BIGINT NOT NULL,
  firmware_id BIGINT NOT NULL,
  version VARCHAR(255) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  FOREIGN KEY (firmware_id) REFERENCES firmwares (id),
  FOREIGN KEY (collection_id) REFERENCES collections (id)
);
