table! {
    club_members (id) {
        id -> Int4,
        user_id -> Int4,
        club_id -> Int4,
        is_moderator -> Text,
    }
}

table! {
    clubs (id) {
        id -> Int4,
        name -> Text,
        body -> Text,
        publish_date -> Timestamptz,
        expiry_date -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Text,
        picture -> Text,
        first_name -> Text,
        last_name -> Text,
        is_admin -> Bool,
    }
}

joinable!(club_members -> clubs (club_id));
joinable!(club_members -> users (user_id));

allow_tables_to_appear_in_same_query!(
    club_members,
    clubs,
    users,
);
