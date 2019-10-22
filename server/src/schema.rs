table! {
    game_hands (id) {
        id -> Int4,
        game_uuid -> Nullable<Varchar>,
        user_uuid -> Nullable<Varchar>,
        card -> Varchar,
        last_updated_at -> Timestamp,
    }
}

table! {
    games (uuid) {
        uuid -> Varchar,
        room_uuid -> Nullable<Varchar>,
        sequence -> Nullable<Int4>,
        title -> Varchar,
        description -> Nullable<Varchar>,
        created_at -> Timestamp,
        last_updated_at -> Timestamp,
    }
}

table! {
    room_players (player_uuid) {
        room_uuid -> Nullable<Varchar>,
        player_uuid -> Varchar,
        joined_at -> Timestamp,
        left_at -> Nullable<Timestamp>,
    }
}

table! {
    rooms (uuid) {
        uuid -> Varchar,
        passphrase -> Nullable<Varchar>,
        card_set -> Array<Text>,
        current_game_uuid -> Nullable<Varchar>,
        owner_uuid -> Varchar,
        created_at -> Timestamp,
        last_updated_at -> Timestamp,
    }
}

table! {
    users (uuid) {
        uuid -> Varchar,
        email -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        created_at -> Timestamp,
        last_updated_at -> Timestamp,
    }
}

joinable!(game_hands -> games (game_uuid));
joinable!(game_hands -> users (user_uuid));
joinable!(room_players -> rooms (room_uuid));
joinable!(room_players -> users (player_uuid));

allow_tables_to_appear_in_same_query!(game_hands, games, room_players, rooms, users,);
