table! {
    administrators (user_id) {
        user_id -> Int8,
    }
}

table! {
    auto_replies (id) {
        id -> Int4,
        server_id -> Int8,
        channel_id -> Int8,
        message -> Text,
        on_join -> Bool,
        delay -> Int4,
        filters -> Nullable<Text>,
    }
}

table! {
    channel_configs (id) {
        id -> Int4,
        server_id -> Int8,
        channel_id -> Int8,
        image_dump_allowed -> Nullable<Bool>,
    }
}

table! {
    delete_all_messages (id) {
        id -> Int4,
        server_id -> Int8,
        channel_id -> Int8,
        after -> Int4,
        exclude -> Bytea,
    }
}

table! {
    log_channels (server_id, channel_id) {
        server_id -> Int8,
        channel_id -> Int8,
    }
}

table! {
    party_finder_configs (server_id, channel_id) {
        server_id -> Int8,
        channel_id -> Int8,
        message_id -> Int8,
        role_id -> Int8,
        emoji -> Text,
        timeout -> Int8,
    }
}

table! {
    presences (id) {
        id -> Int4,
        kind -> Int2,
        content -> Varchar,
    }
}

table! {
    reactions (id) {
        id -> Int4,
        server_id -> Int8,
        channel_id -> Int8,
        message_id -> Int8,
        emoji -> Text,
        role_id -> Int8,
    }
}

table! {
    role_check_times (id) {
        id -> Int4,
        check_id -> Int4,
        user_id -> Int8,
        reminded_at -> Int8,
        kick_after -> Int4,
    }
}

table! {
    roles (role_name) {
        role_name -> Varchar,
    }
}

table! {
    server_configs (id) {
        id -> Int4,
        server_id -> Int8,
        timeout_role -> Nullable<Text>,
    }
}

table! {
    tags (id) {
        id -> Int4,
        user_id -> Int8,
        server_id -> Int8,
        character_id -> Int8,
        character -> Varchar,
        server -> Varchar,
        last_updated -> Int8,
    }
}

table! {
    timeouts (id) {
        id -> Int4,
        user_id -> Int8,
        server_id -> Int8,
        role_id -> Int8,
        seconds -> Int4,
        start -> Int8,
    }
}

table! {
    verifications (id) {
        id -> Int4,
        tag_id -> Int4,
        verified -> Bool,
        verification_string -> Nullable<Varchar>,
    }
}

joinable!(verifications -> tags (tag_id));

allow_tables_to_appear_in_same_query!(
    administrators,
    auto_replies,
    channel_configs,
    delete_all_messages,
    log_channels,
    party_finder_configs,
    presences,
    reactions,
    role_check_times,
    roles,
    server_configs,
    tags,
    timeouts,
    verifications,
);
