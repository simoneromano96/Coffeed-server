use crate::routes::upload;
use crate::utils::{create_token, hash, verify};
use actix_web::{web, Error, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use futures::Future;
use juniper::{
    graphiql::graphiql_source,
    http::{playground::playground_source, GraphQLRequest},
    Executor, FieldResult,
};
use juniper_from_schema::graphql_schema_from_file;
use mongodb::{
    bson, coll::Collection, db::ThreadedDatabase, doc, oid::ObjectId, Client, ThreadedClient,
};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::sync::Arc;
graphql_schema_from_file!("src/schema.graphql");

pub struct Context {
    db_client: Client,
}
impl juniper::Context for Context {}

pub struct Query;
pub struct Mutation;

#[derive(Serialize, Deserialize)]
pub struct Coffee {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub price: f64,
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    pub description: Option<String>,
}

impl CoffeeFields for Coffee {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<juniper::ID> {
        Ok(juniper::ID::new(self.id.to_hex()))
    }
    fn field_name(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.name)
    }
    fn field_price(&self, _: &Executor<'_, Context>) -> FieldResult<&f64> {
        Ok(&self.price)
    }
    fn field_image_url(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.image_url)
    }
    fn field_description(&self, _: &Executor<'_, Context>) -> FieldResult<&Option<String>> {
        Ok(&self.description)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Coffees {
    pub coffees: Vec<Coffee>,
}

impl CoffeesFields for Coffees {
    fn field_coffees(
        &self,
        _: &Executor<'_, Context>,
        _parent: &juniper_from_schema::QueryTrail<Coffee, juniper_from_schema::Walked>,
    ) -> FieldResult<&Vec<Coffee>> {
        Ok(&self.coffees)
    }
}

// #[derive(Serialize, Deserialize)]
// pub struct Jwt {
//     pub jwt: String,
// }
//
// impl JwtFields for Jwt {
//     fn field_jwt(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
//         Ok(&self.jwt)
//     }
// }

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
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(rename = "userType")]
    pub user_type: String,
}

impl UserFields for User {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<juniper::ID> {
        Ok(juniper::ID::new(self.id.to_hex()))
    }
    fn field_username(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.username)
    }
    fn field_email(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.email)
    }
    fn field_password(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.password)
    }
    fn field_user_type(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.user_type)
    }
}

impl Serialize for UpdateCoffeeInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("UpdateCoffeeInput", 4)?;
        // s.serialize_field("_id", &self.id)?; //! Don't serialize ID
        s.serialize_field("name", &self.name)?;
        s.serialize_field("price", &self.price)?;
        s.serialize_field("imageUrl", &self.image_url)?;
        s.serialize_field("description", &self.description)?;
        s.end()
    }
}

// Query resolvers
impl QueryFields for Query {
    //fn field_login(
    //    &self,
    //    executor: &juniper::Executor<'_, Context>,
    //    _parent: &juniper_from_schema::QueryTrail<BaseResponse, juniper_from_schema::Walked>,
    //    data: UserInput,
    //) -> FieldResult<BaseResponse> {
    //    // 1. Get context
    //    let context = executor.context();
    //    // 2. Get the db Connection
    //    let connection: Client = context.db_client.clone();
    //    // 3. Get the db
    //    let database = connection.db("coffeed");
    //    // 4. Get collection
    //    let collection: Collection = database.collection("users");
    //    // 5. Find user by username
    //    let result_document = collection
    //        .find_one(
    //            Some(doc! { "username": data.username, "email": data.email }),
    //            None,
    //        )
    //        .unwrap()
    //        .unwrap();
    //    // 6. Deserialize the document into a Coffee instance
    //    let user: User = bson::from_bson(bson::Bson::Document(result_document))?;
    //    // 7. Verify password
    //    verify(user.password, data.password).unwrap();
    //    // 8. Create token
    //    let token: Jwt = Jwt {
    //        jwt: create_token(user.username, user.email),
    //    };
    //    // 9. Create response
    //    let response: BaseResponse = BaseResponse {
    //        error: false,
    //        status_code: 200,
    //        timestamp: Utc::now().naive_utc(),
    //        message: String::from("Updated successfully"),
    //        data: Some(BaseResponseData::from(token)),
    //    };
    //    Ok(response)
    //}

    // TODO Handle error!
    fn field_coffees(
        &self,
        executor: &Executor<'_, Context>,
        _parent: &juniper_from_schema::QueryTrail<BaseResponse, juniper_from_schema::Walked>,
    ) -> FieldResult<BaseResponse> {
        // 1. Get context
        let context = executor.context();
        // 2. Get the db Connection
        let connection: Client = context.db_client.clone();
        // 3. Get the db
        let database = connection.db("coffeed");
        // 4. Get collection
        let collection: Collection = database.collection("coffees");
        // 6. Find coffees
        let coffees = collection.find(None, None).expect("Document not found");
        // 7. Deserialize the document into a Coffee instance
        let mut result: Coffees = Coffees {
            coffees: Vec::new(),
        };
        for coffee_document in coffees {
            if let Ok(item) = coffee_document {
                let coffee: Coffee = bson::from_bson(bson::Bson::Document(item))?;
                // info!("{:?}", coffee);
                result.coffees.push(coffee);
            }
        }
        // 8. Create response
        let response: BaseResponse = BaseResponse {
            error: false,
            status_code: 200,
            timestamp: Utc::now().naive_utc(),
            message: String::from("Got coffees successfully"),
            data: Some(BaseResponseData::from(result)),
        };

        Ok(response)
    }

    // TODO Handle error!
    fn field_coffee(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _parent: &juniper_from_schema::QueryTrail<BaseResponse, juniper_from_schema::Walked>,
        id: juniper::ID,
    ) -> FieldResult<BaseResponse> {
        // 1. Get context
        let context = executor.context();
        // 2. Get the db Connection
        let connection: Client = context.db_client.clone();
        // 3. Get the db
        let database = connection.db("coffeed");
        // 4. Get collection
        let collection: Collection = database.collection("coffees");
        // 5. Convert objectId
        let oid = ObjectId::with_string(&id).expect("Id not valid");
        // 6. Find coffee
        let result_document = collection
            .find_one(Some(doc! { "_id":  oid }), None)?
            .expect("Document not found");
        // 7. Deserialize the document into a Coffee instance
        let result: Coffee = bson::from_bson(bson::Bson::Document(result_document))?;
        // 8. Create response
        let response: BaseResponse = BaseResponse {
            error: false,
            status_code: 200,
            timestamp: Utc::now().naive_utc(),
            message: String::from("Created successfully"),
            data: Some(BaseResponseData::from(result)),
        };
        Ok(response)
    }
}

// Mutation resolvers
impl MutationFields for Mutation {
    // TODO Handle error!
    fn field_create_coffee(
        &self,
        executor: &Executor<'_, Context>,
        _trail: &QueryTrail<'_, BaseResponse, Walked>,
        data: CoffeeInput,
    ) -> FieldResult<BaseResponse> {
        let new_coffee = Coffee {
            // id: nanoid::simple(),
            id: ObjectId::new().unwrap(),
            name: data.name,
            price: data.price,
            image_url: data.image_url,
            description: data.description,
        };

        // 1. Get context
        let context = executor.context();
        // 2. Get the db Connection
        let connection: Client = context.db_client.clone();
        // 3. Get the db
        let database = connection.db("coffeed");
        // 4. Get collection
        let collection: Collection = database.collection("coffees");
        // 5. Serialize
        let bson = bson::to_bson(&new_coffee)?;
        // 6. Save
        if let bson::Bson::Document(document) = bson {
            collection.insert_one(document, None)?; // Insert into a MongoDB collection
        }
        // 7. Create response
        let response: BaseResponse = BaseResponse {
            error: false,
            status_code: 200,
            timestamp: Utc::now().naive_utc(),
            message: String::from("Created successfully"),
            data: Some(BaseResponseData::from(new_coffee)),
        };

        Ok(response)
    }

    // TODO Make a generic response handler
    fn field_update_coffee(
        &self,
        executor: &Executor<'_, Context>,
        _trail: &QueryTrail<'_, BaseResponse, Walked>,
        data: UpdateCoffeeInput,
    ) -> FieldResult<BaseResponse> {
        // 1. Get context
        let context = executor.context();
        // 2. Get the db Connection
        let connection: Client = context.db_client.clone();
        // 3. Get the db
        let database = connection.db("coffeed");
        // 4. Get collection
        let collection: Collection = database.collection("coffees");
        // 5. Convert objectId
        let oid = ObjectId::with_string(&data.id).expect("Id not valid");
        // 6. Serialize
        let bson = bson::to_bson(&data)?;
        // 7. Prepare a deserialized variable
        let result: Coffee;
        // 8. Base error response
        let mut response: BaseResponse = BaseResponse {
            error: true,
            status_code: 200,
            timestamp: Utc::now().naive_utc(),
            message: String::from("Error"),
            data: None,
        };
        // 9. Update
        if let bson::Bson::Document(document) = bson {
            let document = collection
                .find_one_and_update(doc! {"_id":  oid}, doc! { "$set": document }, None)
                .unwrap()
                .unwrap();
            result = bson::from_bson(bson::Bson::Document(document)).unwrap();

            response = BaseResponse {
                error: true,
                status_code: 200,
                timestamp: Utc::now().naive_utc(),
                message: String::from("Updated successfully"),
                data: Some(BaseResponseData::from(result)),
            };
        };

        Ok(response)
    }

    // TODO Handle error!
    fn field_delete_coffee(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _parent: &juniper_from_schema::QueryTrail<BaseResponse, juniper_from_schema::Walked>,
        id: juniper::ID,
    ) -> FieldResult<BaseResponse> {
        // 1. Get context
        let context = executor.context();
        // 2. Get the db Connection
        let connection: Client = context.db_client.clone();
        // 3. Get the db
        let database = connection.db("coffeed");
        // 4. Get collection
        let collection: Collection = database.collection("coffees");
        // 5. Convert objectId
        // let oid = ObjectId::with_string(&id).expect("Id not valid");
        // 6. Find and delete coffee
        let result: Coffee = bson::from_bson(bson::Bson::Document(
            collection
                .find_one_and_delete(doc! { "_id":  id.to_string() }, None)
                .unwrap()
                .unwrap(),
        ))
        .unwrap();
        // 7. Create response
        let response: BaseResponse = BaseResponse {
            error: false,
            status_code: 200,
            timestamp: Utc::now().naive_utc(),
            message: String::from("Updated successfully"),
            data: Some(BaseResponseData::from(result)),
        };

        Ok(response)
    }
}

fn playground() -> HttpResponse {
    let html = playground_source("http://127.0.0.1:8082/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://127.0.0.1:8082/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn graphql(
    schema: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
    //user: User,
    db_client: web::Data<Client>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let ctx = Context {
        db_client: db_client.get_ref().clone(),
    };

    web::block(move || {
        let res = data.execute(&schema, &ctx);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .map_err(Error::from)
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}

pub fn register(config: &mut web::ServiceConfig) {
    let schema = std::sync::Arc::new(Schema::new(Query, Mutation));

    config
        .data(schema)
        .route("/graphql", web::post().to_async(graphql))
        .route("/playground", web::get().to(playground))
        .route("/graphiql", web::get().to(graphiql))
        .route("/upload", web::post().to_async(upload));
}
