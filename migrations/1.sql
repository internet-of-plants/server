CREATE TABLE IF NOT EXISTS migrations (
    id SMALLINT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

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

CREATE TABLE IF NOT EXISTS binary_updates (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    collection_id BIGINT NOT NULL,
    file_hash VARCHAR(255) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    version VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (collection_id) REFERENCES collections (id)
);
