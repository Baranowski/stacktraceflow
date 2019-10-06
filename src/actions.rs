use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", content = "args")]
pub enum Action {
    Recursive(String),
    Delete(String),
}
