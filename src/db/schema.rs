// @generated automatically by Diesel CLI.

// diesel migration generate --diff-schema <migration_name>
// diesel migration run

diesel::table! {
    posts (id) {
        id -> Int4,
        title -> Text,
        body -> Text,
        published -> Bool,
    }
}
