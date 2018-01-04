table! {
    auto_replies (id) {
        id -> Integer,
        server_id -> Text,
        channel_id -> Text,
        message -> Text,
        on_join -> Bool,
        delay -> Integer,
        filters -> Nullable<Text>,
    }
}

table! {
    channel_configs (id) {
        id -> Integer,
        server_id -> Text,
        channel_id -> Text,
        image_dump_allowed -> Nullable<Bool>,
    }
}

table! {
    delete_all_messages (id) {
        id -> Integer,
        server_id -> Text,
        channel_id -> Text,
        after -> Integer,
        exclude -> Binary,
    }
}

table! {
    edits (id) {
        id -> Integer,
        message_id -> Integer,
        content -> Text,
    }
}

table! {
    messages (id) {
        id -> Integer,
        message_id -> Text,
        channel_id -> Text,
        content -> Text,
    }
}

table! {
    progression (id) {
        id -> Integer,
        tag_id -> Integer,
        xp -> BigInt,
        level -> Integer,
        reputation -> Integer,
        last_xp -> Nullable<Integer>,
    }
}

table! {
    reactions (id) {
        id -> Integer,
        server_id -> Text,
        channel_id -> Text,
        message_id -> Text,
        emoji -> Text,
        role -> Text,
    }
}

table! {
    server_configs (id) {
        id -> Integer,
        server_id -> Text,
        timeout_role -> Nullable<Text>,
    }
}

table! {
    tags (id) {
        id -> Integer,
        user_id -> Text,
        server_id -> Text,
        character_id -> Text,
        character -> Text,
        server -> Text,
        last_updated -> BigInt,
    }
}

table! {
    timeouts (id) {
        id -> Integer,
        user_id -> Text,
        server_id -> Text,
        role_id -> Text,
        seconds -> Integer,
        start -> BigInt,
    }
}

table! {
    verifications (id) {
        id -> Integer,
        tag_id -> Integer,
        verified -> Bool,
        verification_string -> Nullable<Text>,
    }
}

joinable!(edits -> messages (message_id));
joinable!(progression -> tags (tag_id));
joinable!(verifications -> tags (tag_id));

allow_tables_to_appear_in_same_query!(
    auto_replies,
    channel_configs,
    delete_all_messages,
    edits,
    messages,
    progression,
    reactions,
    server_configs,
    tags,
    timeouts,
    verifications,
);
