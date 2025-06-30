// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Int4,
        user_id -> Int4,
        type_ -> Text,
        email -> Nullable<Text>,
        password -> Nullable<Text>,
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
    sessions (id) {
        id -> Text,
        session_token -> Text,
        user_id -> Int4,
        expires -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    verification_tokens (identifier, token) {
        identifier -> Text,
        token -> Text,
        expires -> Timestamp,
    }
}

diesel::joinable!(accounts -> users (user_id));
diesel::joinable!(posts -> users (author_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    posts,
    sessions,
    users,
    verification_tokens,
);
