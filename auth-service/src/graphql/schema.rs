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

pub struct Context {}

impl juniper::Context for Context {}

pub struct Query;
pub struct Mutation;

//#[derive(Serialize, Deserialize)]
pub struct BaseResponse {
    pub error: bool,
    pub status_code: i32,
    pub timestamp: NaiveDateTime,
    pub message: String,
    pub data: Option<BaseResponseData>,
}

impl BaseResponseFields for BaseResponse {
    fn field_error(&self, _: &Executor<'_, Context>) -> FieldResult<&bool> {
        Ok(&self.error)
    }
    fn field_status_code(&self, _: &Executor<'_, Context>) -> FieldResult<&i32> {
        Ok(&self.status_code)
    }
    fn field_timestamp(&self, _: &Executor<'_, Context>) -> FieldResult<&NaiveDateTime> {
        Ok(&self.timestamp)
    }
    fn field_message(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.message)
    }
    fn field_data(
        &self,
        _: &Executor<'_, Context>,
        _parent: &juniper_from_schema::QueryTrail<BaseResponseData, juniper_from_schema::Walked>,
    ) -> FieldResult<&Option<BaseResponseData>> {
        Ok(&self.data)
    }
}

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

impl QueryFields for Query {
    fn field_query_test(
        &self,
        _executor: &Executor<'_, Context>,
        _trail: &juniper_from_schema::QueryTrail<BaseResponse, juniper_from_schema::Walked>,
    ) -> FieldResult<BaseResponse> {
        let user_type: UserType = UserType {
            id: ObjectId::new().unwrap(),
            name: String::from(""),
            grants: vec![],
        };

        let response: BaseResponse = BaseResponse {
            error: false,
            status_code: 200,
            timestamp: Utc::now().naive_utc(),
            message: String::from("Created successfully"),
            data: Some(BaseResponseData::from(user_type)),
        };

        Ok(response)
    }
}

impl MutationFields for Mutation {
    fn field_mutation_test(
        &self,
        _executor: &Executor<'_, Context>,
        _trail: &juniper_from_schema::QueryTrail<'_, BaseResponse, juniper_from_schema::Walked>,
    ) -> FieldResult<BaseResponse> {
        let user_type: UserType = UserType {
            id: ObjectId::new().unwrap(),
            name: String::from(""),
            grants: vec![],
        };

        let response: BaseResponse = BaseResponse {
            error: false,
            status_code: 200,
            timestamp: Utc::now().naive_utc(),
            message: String::from("Created successfully"),
            data: Some(BaseResponseData::from(user_type)),
        };

        Ok(response)
    }
}
