use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}
