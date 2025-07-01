use diesel::prelude::*;
use serde::Deserialize;

#[derive(Queryable, Selectable, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::db::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
    pub author_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::db::schema::posts)]
pub struct NewPost {
    pub title: String,
    pub body: String,
    #[serde(default = "default_published")]
    pub published: Option<bool>,
    pub author_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
}

fn default_published() -> Option<bool> {
    Some(true)
}

impl Post {
    pub async fn get_published() -> Vec<Post> {
        use crate::db::schema::posts::dsl::*;
        let mut connection = crate::db::db_utils::establish_pool()
            .get()
            .expect("Failed to get DB connection from pool");
        let results: Vec<Post> = posts
            .filter(diesel::ExpressionMethods::eq(published, true))
            .limit(5)
            .select(Post::as_select())
            .load(&mut connection)
            .expect("Error loading posts");
        println!("Displaying {} posts", results.len());
        results
    }

    pub fn create(
        conn: &mut diesel::PgConnection,
        title: &str,
        body: &str,
        author_id: &Option<i32>,
        created_at_value: chrono::NaiveDateTime,
    ) -> Option<Post> {
        let new_post: NewPost = NewPost::new(
            title.to_string(),
            body.to_string(),
            Some(true),
            *author_id,
            created_at_value,
        );
        println!("Creating post: {new_post:?}");
        let result = diesel::insert_into(crate::db::schema::posts::table)
            .values(&new_post)
            .returning(Post::as_returning())
            .get_result(conn);
        match result {
            Ok(post) => {
                println!("Inserted post: {post:?}");
                Some(post)
            }
            Err(e) => {
                println!("Diesel insert error: {e}");
                None
            }
        }
    }
}

impl NewPost {
    pub fn new(
        title: String,
        body: String,
        published: Option<bool>,
        author_id: Option<i32>,
        created_at_value: chrono::NaiveDateTime,
    ) -> Self {
        NewPost {
            title,
            body,
            published,
            author_id,
            created_at: created_at_value,
        }
    }
}
