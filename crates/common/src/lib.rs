pub use serde;
use serde::{Deserialize, Serialize};
pub use serde_json;
#[derive(Clone, Serialize, Deserialize)]
pub enum Instruction {
    AddPhoneNumber { key: String, number: String },
    DeleteUser { key: String },
    EditNumber { key: String, number: String },
    GetAllUsers,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Response {
    Fail { message: String },
    Number { number: String },
    AllUsers(Vec<(String, String)>),
    Success,
}
