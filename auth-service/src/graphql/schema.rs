use actix_web::{web, Error, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use futures::Future;
use juniper::{http::GraphQLRequest, Executor, FieldResult};
use juniper_from_schema::graphql_schema_from_file;
use mongodb::{coll::Collection, db::ThreadedDatabase, doc, oid::ObjectId, Client, ThreadedClient};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::sync::Arc;

graphql_schema_from_file!("src/graphql/schema.graphql");

pub struct Context {
    db_client: Client,
}
impl juniper::Context for Context {}

pub struct Query;
pub struct Mutation;

#[derive(Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub email: String,
    pub username: String,
    pub password: String,
    #[serde(rename = "userType")]
    pub user_type: ObjectId,
}

impl UserFields for User {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<juniper::ID> {
        Ok(juniper::ID::new(self.id.to_hex()))
    }
    fn field_email(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.email)
    }
    fn field_username(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.username)
    }
    fn field_password(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.password)
    }
    fn field_user_type(
        &self,
        executor: &'_ Executor<'_, Context>,
        trail: &'_ juniper_from_schema::QueryTrail<'_, UserType, juniper_from_schema::Walked>,
    ) -> FieldResult<UserType> {
        // Get ID from self
        let user_type_id: ObjectId = self.user_type.clone();

        // Solve userType
        let user_type: UserType = UserType {
            id: user_type_id,
            name: "test".to_string(),
            grants: vec!["test".to_string()],
        };

        Ok(user_type)
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserType {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub grants: Vec<String>,
}

impl UserTypeFields for UserType {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<juniper::ID> {
        Ok(juniper::ID::new(self.id.to_hex()))
    }
    fn field_name(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.name)
    }
    fn field_grants(&self, _: &Executor<'_, Context>) -> FieldResult<&Vec<String>> {
        Ok(&self.grants)
    }
}
