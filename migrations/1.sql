CREATE TABLE IF NOT EXISTS migrations (
    id SMALLINT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL UNIQUE,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    CHECK (email <> '' AND username <> '')
);

CREATE INDEX IF NOT EXISTS users_email ON users (email);

CREATE TABLE IF NOT EXISTS plants (
    id BIGINT PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    owner_id BIGINT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    FOREIGN KEY (owner_id) REFERENCES users (id),
    UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    air_temperature_celsius SMALLINT NOT NULL,
    air_humidity_percentage SMALLINT NOT NULL,
    soil_temperature_celsius SMALLINT NOT NULL,
    soil_resistivity_raw SMALLINT NOT NULL,
    plant_id BIGINT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    FOREIGN KEY (plant_id) REFERENCES plants (id)
);

CREATE TABLE IF NOT EXISTS errors (
    id BIGSERIAL PRIMARY KEY,
    plant_id BIGINT NOT NULL,
    error TEXT NOT NULL,
    is_solved BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    FOREIGN KEY (plant_id) REFERENCES plants (id)
);

CREATE TABLE IF NOT EXISTS authentications (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    token VARCHAR(255) NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    FOREIGN KEY (user_id) REFERENCES users (id)
);
