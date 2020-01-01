use refinery::embed_migrations;
/*
mod V1__create_user_types;
mod V2__create_users;

use V1__create_user_types::migration;
*/

embed_migrations!("src/migrations");
