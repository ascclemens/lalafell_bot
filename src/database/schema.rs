table! {
    auto_replies (id) {
        id -> Int4,
        server_id -> Text,
        channel_id -> Text,
        message -> Text,
        on_join -> Bool,
        delay -> Int4,
        filters -> Nullable<Text>,
    }
}

table! {
    channel_configs (id) {
        id -> Int4,
        server_id -> Text,
        channel_id -> Text,
        image_dump_allowed -> Nullable<Bool>,
    }
}

table! {
    delete_all_messages (id) {
        id -> Int4,
        server_id -> Text,
        channel_id -> Text,
        after -> Int4,
        exclude -> Bytea,
    }
}

table! {
    reactions (id) {
        id -> Int4,
        server_id -> Text,
        channel_id -> Text,
        message_id -> Text,
        emoji -> Text,
        role -> Text,
    }
}

table! {
    role_check_times (id) {
        id -> Int4,
        check_id -> Int4,
        user_id -> Text,
        reminded_at -> Int8,
        kick_after -> Int4,
    }
}

table! {
    server_configs (id) {
        id -> Int4,
        server_id -> Text,
        timeout_role -> Nullable<Text>,
    }
}

table! {
    tags (id) {
        id -> Int4,
        user_id -> Text,
        server_id -> Text,
        character_id -> Text,
        character -> Varchar,
        server -> Varchar,
        last_updated -> Int8,
    }
}

table! {
    timeouts (id) {
        id -> Int4,
        user_id -> Text,
        server_id -> Text,
        role_id -> Text,
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
    auto_replies,
    channel_configs,
    delete_all_messages,
    reactions,
    role_check_times,
    server_configs,
    tags,
    timeouts,
    verifications,
);
