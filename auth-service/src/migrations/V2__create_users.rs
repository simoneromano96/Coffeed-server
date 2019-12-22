use barrel::{backend::MySql, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();
    m.create_table("users", |t| {
        t.add_column("id", types::varchar(32).primary(true));
        t.add_column("username", types::varchar(255));
        t.add_column("email", types::varchar(255));
        t.add_column("password", types::varchar(255));
        t.add_column("user_type", types::foreign("user_types", "id"));
    });

    m.make::<MySql>()
}
