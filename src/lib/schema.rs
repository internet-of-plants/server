table! {
    events (id) {
        id -> Int8,
        plant_id -> Int4,
        air_temperature_celsius -> Int2,
        air_humidity_percentage -> Int2,
        soil_temperature_celsius -> Int2,
        soil_resistivity -> Int2,
        light -> Int2,
        device_timestamp -> Int4,
        timestamp -> Int8,
    }
}

table! {
    plants (id) {
        id -> Int4,
        name -> Varchar,
        type_id -> Int4,
        user_id -> Int4,
        last_event_id -> Nullable<Int8>,
        timestamp -> Int8,
    }
}

table! {
    plant_types (id) {
        id -> Int4,
        name -> Varchar,
        slug -> Varchar,
        filename -> Bpchar,
        user_id -> Int4,
        timestamp -> Int8,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        timestamp -> Int8,
    }
}

joinable!(plant_types -> users (user_id));
joinable!(plants -> plant_types (type_id));
joinable!(plants -> users (user_id));

allow_tables_to_appear_in_same_query!(
    events,
    plants,
    plant_types,
    users,
);
