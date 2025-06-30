// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Int4,
        author_id -> Nullable<Int4>,
        created_at -> Timestamp,
        title -> Text,
        body -> Text,
        published -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Text,

        created_at -> Timestamp,
        accounts -> Integer
    }
}

diesel::table! {
    accounts (id) {
        id -> Integer,
        user_id -> Integer,
        type_ -> Text, // Utiliser ce champ comme enum côté Rust
        provider -> Nullable<Text>,
        provider_account_id -> Nullable<Text>,
        refresh_token -> Nullable<Text>,
        access_token -> Nullable<Text>,
        expires_at -> Nullable<Int4>,
        token_type -> Nullable<Text>,
        scope -> Nullable<Text>,
        id_token -> Nullable<Text>,
        session_state -> Nullable<Text>,
        refresh_token_expires_in -> Nullable<Int4>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Text,
        session_token -> Text,
        user_id -> Text,
        expires -> Timestamp,
    }
}

diesel::table! {
    verification_tokens (identifier, token) {
        identifier -> Text,
        token -> Text,
        expires -> Timestamp,
    }
}

// Relations
diesel::joinable!(accounts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    posts,
    users,
    accounts,
    sessions,
    verification_tokens,
);
