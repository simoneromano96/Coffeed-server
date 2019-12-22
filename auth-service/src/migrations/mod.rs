use refinery::include_migration_mods;
/*
mod V1__create_user_types;
mod V2__create_users;

use V1__create_user_types::migration;
*/

include_migration_mods!("src/migrations");
