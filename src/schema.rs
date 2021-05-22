table! {
    todos (id) {
        id -> Int4,
        text -> Varchar,
        done -> Bool,
        user_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(todos, users,);
