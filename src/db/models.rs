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
}

#[derive(Insertable, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::db::schema::posts)]
pub struct NewPost {
    pub title: String,
    pub body: String,
    #[serde(default = "default_published")]
    pub published: Option<bool>,
}

fn default_published() -> Option<bool> {
    Some(true)
}

impl Post {
    pub async fn get_published() -> Vec<Post> {
        use crate::db::schema::posts::dsl::*;
        let connection: &mut diesel::PgConnection =
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

    pub fn create(conn: &mut diesel::PgConnection, title: &str, body: &str) -> Option<Post> {
        let new_post: NewPost = NewPost::new(title.to_string(), body.to_string(), Some(true));
        println!("Creating post: {:?}", new_post);
        let result = diesel::insert_into(crate::db::schema::posts::table)
            .values(&new_post)
            .returning(Post::as_returning())
            .get_result(conn);
        match result {
            Ok(post) => {
                println!("Inserted post: {:?}", post);
                Some(post)
            }
            Err(e) => {
                println!("Diesel insert error: {}", e);
                None
            }
        }
    }
}

impl NewPost {
    pub fn new(title: String, body: String, published: Option<bool>) -> Self {
        NewPost {
            title,
            body,
            published,
        }
    }
}
