// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Int4,
        author_id -> Int4,
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
        password -> Text,
        email -> Nullable<Text>,
        created_at -> Timestamp,

        posts -> Array<Int4>,
    }
}
