create table `users`
(
    id        varchar(32) primary key,
    username  varchar(255),
    email     varchar(255) unique,
    password  varchar(255),
    user_type varchar(32)
);

alter table `users`
    add constraint user_type_fk foreign key (user_type) references `user_types` (id) ON DELETE CASCADE
        ON UPDATE CASCADE
;