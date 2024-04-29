use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct Response<Payload> {
    pub(crate) heos: Header,
    pub(crate) payload: Payload,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Header {
    pub(crate) command: String,
    pub(crate) result: HeosResult,
    pub(crate) message: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum HeosResult {
    Success,
    Fail,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Player {
    pub(crate) name: String,
    pub(crate) pid: i64,
    pub(crate) model: String,
    pub(crate) version: String,
    pub(crate) network: String,
    pub(crate) lineout: u8,
    pub(crate) serial: String,
}
