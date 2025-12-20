use crate::{helpers::FromRequest, req::body::RequestBodyContent};

pub struct JsonBody<T> {
    pub data: T,
}

impl<T: FromJson> FromRequest for JsonBody<T> {
    type Error = String;

    fn from_request(req: &crate::req::HttpRequest) -> Result<Self, Self::Error> {
        let body = &req.body.content;

        Ok(Self {
            data: T::from_json(body)?,
        })
    }
}

pub trait FromJson: Sized {
    fn from_json(data: &RequestBodyContent) -> Result<Self, String>;
}
