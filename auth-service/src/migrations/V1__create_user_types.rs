use barrel::{backend::MySql, types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();
    m.create_table("user_types", |t| {
        t.add_column("id", types::varchar(32).primary(true));
        t.add_column("name", types::varchar(255));
        // t.add_column("grants", types::array(&types::varchar(8)));
        t.add_column(
            "grants",
            types::custom("SET('create','read','update','delete')"),
        );
    });

    m.make::<MySql>()
}
