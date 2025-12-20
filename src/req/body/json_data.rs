use std::ops::Deref;
use crate::{helpers::FromRequest, req::body::RequestBodyContent};

pub struct JsonBody<T>(T);

impl<T: FromJson> FromRequest for JsonBody<T> {
    type Error = String;

    fn from_request(req: &crate::req::HttpRequest) -> Result<Self, Self::Error> {
        let body = &req.body.content;
        Ok(Self(T::from_json(body)?))
    }
}

impl<T> Deref for JsonBody<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait FromJson: Sized {
    fn from_json(data: &RequestBodyContent) -> Result<Self, String>;
}
