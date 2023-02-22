use crate::user::User;

pub struct Group {
    pub uid: String,
    pub name: String,
    pub members: Vec<User>,
}

