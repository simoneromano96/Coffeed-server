use actix_web::{web, Error, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use futures::Future;
use juniper::{http::GraphQLRequest, Executor, FieldResult};
use juniper_from_schema::graphql_schema_from_file;
use mongodb::{
    bson, coll::Collection, db::ThreadedDatabase, doc, oid::ObjectId, Client, ThreadedClient,
};
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
    #[serde(rename = "userTypeId")]
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
    fn field_user_type(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&String::from("asd"))
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserType {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub grants: Vec<String>,
}
