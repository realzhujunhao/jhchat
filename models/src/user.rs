#[derive(Debug)]
pub struct User {
    pub name: String,
    //TODO OTHER INFO
}

impl User {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

