use chrono::NaiveDateTime;
use diesel::{associations::Identifiable, deserialize::Queryable, prelude::Insertable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Selectable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[diesel(table_name = crate::db::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub joined_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Selectable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[diesel(table_name = crate::db::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct UserIdWithPassword {
    pub id: i32,
    pub password: String,
}

#[derive(Serialize, Deserialize, Insertable, Debug, Clone, PartialEq)]
#[diesel(table_name = crate::db::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreatingUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Serialize, Deserialize, Selectable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[diesel(primary_key(user_id, token))]
#[diesel(table_name = crate::db::schema::user_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct UserSession {
    pub user_id: i32,
    pub token: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Insertable, Debug, Clone, PartialEq)]
#[diesel(table_name = crate::db::schema::user_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreatingUserSession<'a> {
    pub user_id: i32,
    pub token: &'a str,
}