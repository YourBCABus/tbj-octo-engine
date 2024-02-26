mod _period_teacher;
mod _teachers;
mod _report_to;

pub use _period_teacher::*;
pub use _teachers::*;
pub use _report_to::*;

use crate::env::{CLIENT_ID, CLIENT_SECRET};

fn post_req_auth(url: &str) -> reqwest::RequestBuilder {
    reqwest::Client::new().post(url)
        .header("Client-Id", CLIENT_ID.as_str())
        .header("Client-Secret", CLIENT_SECRET.as_str())
}
