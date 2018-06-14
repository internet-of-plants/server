CREATE TABLE users (
    id            SERIAL PRIMARY KEY,
    username      VARCHAR(255) NOT NULL,
    email         VARCHAR(255) NOT NULL,
    password_hash VARCHAR(204) NOT NULL,
    timestamp     BIGINT       NOT NULL DEFAULT extract(epoch from now()),
    CHECK (username <> '' AND email <> ''),
    UNIQUE (username),
    UNIQUE (email)
);
CREATE INDEX users_username_index on users using hash(username);
INSERT INTO users (username, email, password_hash) VALUES ('Deleted', 'deleted@example.com', '');

CREATE TABLE plant_types (
    id        SERIAL PRIMARY KEY,
    name      VARCHAR(255) NOT NULL,
    slug      VARCHAR(20)  NOT NULL,
    filename  CHAR(20)     NOT NULL,
    user_id   INTEGER      NOT NULL,
    timestamp BIGINT       NOT NULL DEFAULT extract(epoch from now()),
    FOREIGN KEY (user_id) REFERENCES users (id),
    CHECK (name <> '' AND slug <> '' AND filename <> ''),
    UNIQUE (slug)
);
CREATE INDEX plant_types_slug_index on plant_types using hash(slug);

CREATE TABLE plants (
    id            SERIAL PRIMARY KEY,
    name          VARCHAR(255) NOT NULL,
    type_id       INTEGER      NOT NULL,
    user_id       INTEGER      NOT NULL,
    last_event_id BIGINT,
    timestamp     BIGINT       NOT NULL DEFAULT extract(epoch from now()),
    CHECK (name <> ''),
    FOREIGN KEY (type_id) REFERENCES plant_types (id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE events (
    id                       BIGSERIAL PRIMARY KEY,
    plant_id                 INTEGER  NOT NULL,
    air_temperature_celsius  SMALLINT NOT NULL,
    air_humidity_percentage  SMALLINT NOT NULL,
    soil_temperature_celsius SMALLINT NOT NULL,
    soil_resistivity         SMALLINT NOT NULL,
    light                    SMALLINT NOT NULL,
    device_timestamp         INTEGER  NOT NULL,
    timestamp                BIGINT   NOT NULL DEFAULT extract(epoch from now()),
    FOREIGN KEY (plant_id) REFERENCES plants (id) ON DELETE CASCADE
);

ALTER TABLE plants ADD FOREIGN KEY (last_event_id) REFERENCES events (id) ON DELETE SET NULL;

CREATE FUNCTION cache_last_event() RETURNS trigger AS $$ BEGIN
UPDATE plants
SET last_event_id = NEW.id
WHERE plants.id = NEW.plant_id;
RETURN NEW;
END; $$ LANGUAGE plpgsql;

CREATE TRIGGER last_event_cache AFTER INSERT ON events
    FOR EACH ROW EXECUTE PROCEDURE cache_last_event();
