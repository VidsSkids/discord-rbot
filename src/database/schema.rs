// @generated automatically by Diesel CLI.

diesel::table! {
    airtable (id) {
        id -> Int4,
        aid -> Varchar,
        content -> Varchar,
        created_time -> Nullable<Timestamp>,
    }
}

diesel::table! {
    events (id) {
        id -> Int4,
        author -> Int8,
        content -> Varchar,
        channel -> Int8,
        trigger_date -> Timestamp,
        is_triggered -> Bool,
    }
}

diesel::table! {
    invites (id) {
        id -> Int4,
        code -> Varchar,
        actionrole -> Nullable<Int8>,
        actionchannel -> Nullable<Int8>,
        used_count -> Int4,
    }
}

diesel::table! {
    messages (id) {
        id -> Int8,
        author -> Int8,
        content -> Varchar,
        channel -> Int8,
        date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    messages_edits (id) {
        id -> Int4,
        author -> Int8,
        content -> Varchar,
        channel -> Int8,
        date -> Nullable<Timestamp>,
        parrent_message_id -> Int8,
    }
}

diesel::table! {
    notes (id) {
        id -> Int4,
        name -> Text,
        date -> Timestamp,
        note -> Nullable<Text>,
    }
}

diesel::table! {
    projects (id) {
        id -> Int4,
        message_id -> Int8,
        channel_id -> Int8,
        codex -> Varchar,
        client -> Varchar,
        lead -> Varchar,
        deadline -> Varchar,
        description -> Varchar,
        contexte -> Varchar,
        created_at -> Timestamp,
        pinned_message_id -> Nullable<Int8>,
    }
}

diesel::table! {
    storage (id) {
        id -> Int4,
        datatype -> Int8,
        dataid -> Nullable<Int8>,
        data -> Varchar,
        date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        discordid -> Int8,
        role -> Varchar,
    }
}

diesel::table! {
    warnings (id) {
        id -> Int4,
        name -> Text,
        email -> Text,
        discord_id -> Text,
        warning_date -> Timestamp,
        event_name -> Nullable<Text>,
        warning_reason -> Int4,
    }
}

diesel::joinable!(messages_edits -> messages (parrent_message_id));

diesel::allow_tables_to_appear_in_same_query!(
    airtable,
    events,
    invites,
    messages,
    messages_edits,
    notes,
    projects,
    storage,
    users,
    warnings,
);
