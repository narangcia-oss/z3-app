use diesel::prelude::*;
use serde::Deserialize;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Insertable, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::db::schema::posts)]
pub struct NewPost {
    pub title: String,
    pub body: String,
}

impl Post {
    pub async fn get_published() -> Vec<Post> {
        use crate::db::schema::posts::dsl::*;
        let connection: &mut diesel::SqliteConnection =
            &mut crate::db::db_utils::establish_connection();
        let results: Vec<Post> = posts
            .filter(diesel::ExpressionMethods::eq(published, true))
            .limit(5)
            .select(Post::as_select())
            .load(connection)
            .expect("Error loading posts");
        println!("Displaying {} posts", results.len());
        results
    }

    pub fn create(conn: &mut diesel::SqliteConnection, title: &str, body: &str) -> Option<Post> {
        let new_post = NewPost { title: title.into(), body: body.into() };
        diesel::insert_into(crate::db::schema::posts::table)
            .values(&new_post)
            .returning(Post::as_returning())
            .get_result(conn)
            .ok()
    }
}
