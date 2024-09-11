use warp::reject::Reject;
#[derive(Debug)]
pub struct BadRequest {
    message: String
}

impl BadRequest {
    pub fn new(msg : &str) -> Self {
        Self { message: msg.to_string() }
    }
}
impl Reject for BadRequest {
}