use crate::db::schema::accounts;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Account model for handling authentication.
///
/// The `type_` field determines the authentication method:
/// - "email" for email/password authentication
/// - "oauth" for OAuth providers (TODO: implement OAuth support)
///
/// For email authentication: email and password fields are used
/// For OAuth authentication: provider, provider_account_id, and token fields are used
#[derive(Queryable, Selectable, Clone, Serialize, Deserialize, Debug)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub id: i32,
    pub user_id: i32,
    /// Authentication type: "email" for email/password, "oauth" for OAuth (TODO)
    pub type_: String,

    // Email/password authentication fields
    pub email: Option<String>,
    pub password: Option<String>,

    // OAuth authentication fields (TODO: implement OAuth support)
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub expires_at: Option<i32>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
    pub session_state: Option<String>,
    pub refresh_token_expires_in: Option<i32>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = accounts)]
pub struct NewAccount {
    pub user_id: i32,
    pub type_: String,
    pub email: Option<String>,
    pub password: Option<String>,
    pub provider: Option<String>,
    pub provider_account_id: Option<String>,
}

impl Account {
    /// Create a new email/password account
    pub fn create_email_account(
        conn: &mut diesel::PgConnection,
        user_id: i32,
        email: String,
        password_hash: String,
    ) -> Result<Account, diesel::result::Error> {
        let new_account = NewAccount {
            user_id,
            type_: "email".to_string(),
            email: Some(email),
            password: Some(password_hash),
            provider: None,
            provider_account_id: None,
        };

        diesel::insert_into(accounts::table)
            .values(&new_account)
            .returning(Account::as_returning())
            .get_result(conn)
    }

    /// Find account by email for authentication
    pub fn find_by_email(
        conn: &mut diesel::PgConnection,
        email: &str,
    ) -> Result<Account, diesel::result::Error> {
        accounts::table
            .filter(accounts::email.eq(email))
            .filter(accounts::type_.eq("email"))
            .select(Account::as_select())
            .first(conn)
    }
}
