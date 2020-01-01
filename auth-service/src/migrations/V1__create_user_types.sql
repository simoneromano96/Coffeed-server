create table `user_types` (
    id varchar(32) primary key,
    name varchar(255) unique,
    grants set('create','read','update','delete')
);