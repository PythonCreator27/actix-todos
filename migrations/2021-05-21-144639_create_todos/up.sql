create table todos (
    id serial primary key,
    text varchar not null,
    done boolean not null default false
);

create table users (
    id serial primary key,
    username varchar unique not null,
    password varchar not null
);

alter table todos add column user_id integer not null;
-- foreign key (user_id) references user (id) on delete cascade