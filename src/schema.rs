table! {
    events (id) {
        id -> Int8,
        plant_id -> Int4,
        air_temperature_celsius -> Int2,
        air_humidity_percentage -> Int2,
        soil_temperature_celsius -> Int2,
        soil_resistivity -> Int2,
        light -> Int2,
        timestamp -> Int8,
    }
}

table! {
    plants (id) {
        id -> Int4,
        type_slug -> Nullable<Bpchar>,
        user_id -> Int4,
    }
}

table! {
    plant_types (id) {
        id -> Int4,
        name -> Bpchar,
        slug -> Bpchar,
        user_id -> Nullable<Int4>,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Bpchar,
        password_hash -> Bpchar,
    }
}

joinable!(events -> plants (plant_id));
joinable!(plant_types -> users (user_id));
joinable!(plants -> users (user_id));

allow_tables_to_appear_in_same_query!(
    events,
    plants,
    plant_types,
    users,
);
