CREATE TABLE users (
    id            SERIAL PRIMARY KEY,
    email         CHAR(255) NOT NULL,
    password_hash CHAR(255) NOT NULL
);

CREATE TABLE plant_types (
    id      SERIAL PRIMARY KEY,
    name    CHAR(255) NOT NULL,
    slug    CHAR(255) NOT NULL,
    user_id INTEGER,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE SET NULL,
    UNIQUE (slug)
);

CREATE TABLE plants (
    id        SERIAL PRIMARY KEY,
    type_slug CHAR(255),
    user_id   INTEGER NOT NULL,
    FOREIGN KEY (type_slug) REFERENCES plant_types (slug) ON DELETE SET NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE events (
    id                       BIGSERIAL PRIMARY KEY,
    plant_id                 INTEGER NOT NULL,
    air_temperature_celsius  SMALLINT NOT NULL,
    air_humidity_percentage  SMALLINT NOT NULL,
    soil_temperature_celsius SMALLINT NOT NULL,
    soil_resistivity         SMALLINT NOT NULL,
    light                    SMALLINT NOT NULL,
    timestamp                BIGINT NOT NULL,
    FOREIGN KEY (plant_id) REFERENCES plants (id) ON DELETE CASCADE,
    CHECK (air_humidity_percentage > 0 AND light > 0 AND timestamp > 0)
);
