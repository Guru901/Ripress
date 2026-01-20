use crate::req::{
    body::{RequestBodyContent, RequestBodyType, TextData},
    HttpRequest,
};
use tokio::io::{AsyncRead, AsyncWrite};

impl AsyncWrite for HttpRequest {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let this = self.get_mut();

        let new_bytes = bytes::Bytes::copy_from_slice(buf);

        match &mut this.body.content {
            RequestBodyContent::BINARY(existing_bytes) => {
                let mut combined = existing_bytes.to_vec();
                combined.extend_from_slice(buf);
                this.body.content = RequestBodyContent::BINARY(combined.into());
            }
            RequestBodyContent::BinaryWithFields(existing_bytes, form_data) => {
                let mut combined = existing_bytes.to_vec();
                combined.extend_from_slice(buf);
                this.body.content =
                    RequestBodyContent::BinaryWithFields(combined.into(), form_data.clone());
            }
            RequestBodyContent::TEXT(text_data) => {
                if let Ok(new_text) = String::from_utf8(buf.to_vec()) {
                    let existing_text = text_data.as_str_lossy();
                    let combined_text = format!("{}{}", existing_text, new_text);
                    this.body.content = RequestBodyContent::TEXT(TextData::new(combined_text));
                } else {
                    let mut combined = text_data.as_bytes().to_vec();
                    combined.extend_from_slice(buf);
                    this.body.content = RequestBodyContent::BINARY(combined.into());
                    this.body.content_type = RequestBodyType::BINARY;
                }
            }
            RequestBodyContent::JSON(json_value) => {
                let json_str = json_value.to_string();
                let mut combined = json_str.as_bytes().to_vec();
                combined.extend_from_slice(buf);
                if let Ok(combined_text) = String::from_utf8(combined.clone()) {
                    this.body.content = RequestBodyContent::TEXT(TextData::new(combined_text));
                    this.body.content_type = RequestBodyType::TEXT;
                } else {
                    this.body.content = RequestBodyContent::BINARY(combined.into());
                    this.body.content_type = RequestBodyType::BINARY;
                }
            }
            RequestBodyContent::FORM(form_data) => {
                let form_str = form_data.to_string();
                let mut combined = form_str.as_bytes().to_vec();
                combined.extend_from_slice(buf);
                if let Ok(combined_text) = String::from_utf8(combined.clone()) {
                    this.body.content = RequestBodyContent::TEXT(TextData::new(combined_text));
                    this.body.content_type = RequestBodyType::TEXT;
                } else {
                    this.body.content = RequestBodyContent::BINARY(combined.into());
                    this.body.content_type = RequestBodyType::BINARY;
                }
            }
            RequestBodyContent::EMPTY => {
                this.body.content = RequestBodyContent::BINARY(new_bytes);
                this.body.content_type = RequestBodyType::BINARY;
            }
        }

        std::task::Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

impl AsyncRead for HttpRequest {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();

        let body_bytes = match &this.body.content {
            RequestBodyContent::TEXT(text_data) => text_data.as_bytes().to_vec(),
            RequestBodyContent::JSON(json_value) => serde_json::to_vec(json_value)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
            RequestBodyContent::FORM(form_data) => form_data.to_string().as_bytes().to_vec(),
            RequestBodyContent::BINARY(bytes) => {
                let bytes_to_copy = std::cmp::min(buf.remaining(), bytes.len());
                buf.put_slice(&bytes[..bytes_to_copy]);

                if bytes_to_copy == bytes.len() {
                    this.body.content = RequestBodyContent::EMPTY;
                    this.body.content_type = RequestBodyType::EMPTY;
                } else {
                    this.body.content = RequestBodyContent::BINARY(bytes.slice(bytes_to_copy..));
                }
                return std::task::Poll::Ready(Ok(()));
            }
            RequestBodyContent::BinaryWithFields(bytes, _form_data) => bytes.to_vec(),
            RequestBodyContent::EMPTY => Vec::new(),
        };

        if !body_bytes.is_empty() {
            let bytes_to_copy = std::cmp::min(buf.remaining(), body_bytes.len());
            let start_pos = 0;
            let end_pos = bytes_to_copy;

            buf.put_slice(&body_bytes[start_pos..end_pos]);

            if bytes_to_copy == body_bytes.len() {
                this.body.content = RequestBodyContent::EMPTY;
                this.body.content_type = RequestBodyType::EMPTY;
            } else {
                let remaining_bytes = body_bytes[end_pos..].to_vec();
                match &this.body.content {
                    RequestBodyContent::TEXT(_) => {
                        if let Ok(remaining_text) = String::from_utf8(remaining_bytes.clone()) {
                            this.body.content =
                                RequestBodyContent::TEXT(TextData::new(remaining_text));
                        } else {
                            this.body.content =
                                RequestBodyContent::BINARY(remaining_bytes.clone().into());
                            this.body.content_type = RequestBodyType::BINARY;
                        }
                    }
                    RequestBodyContent::JSON(_) => {
                        if let Ok(remaining_text) = String::from_utf8(remaining_bytes.clone()) {
                            this.body.content =
                                RequestBodyContent::TEXT(TextData::new(remaining_text));
                            this.body.content_type = RequestBodyType::TEXT;
                        } else {
                            this.body.content =
                                RequestBodyContent::BINARY(remaining_bytes.clone().into());
                            this.body.content_type = RequestBodyType::BINARY;
                        }
                    }
                    RequestBodyContent::FORM(_) => {
                        if let Ok(remaining_text) = String::from_utf8(remaining_bytes.clone()) {
                            this.body.content =
                                RequestBodyContent::TEXT(TextData::new(remaining_text));
                            this.body.content_type = RequestBodyType::TEXT;
                        } else {
                            this.body.content =
                                RequestBodyContent::BINARY(remaining_bytes.clone().into());
                            this.body.content_type = RequestBodyType::BINARY;
                        }
                    }
                    RequestBodyContent::BINARY(_) => {
                        this.body.content = RequestBodyContent::BINARY(remaining_bytes.into());
                    }
                    RequestBodyContent::BinaryWithFields(_, form_data) => {
                        this.body.content = RequestBodyContent::BinaryWithFields(
                            remaining_bytes.into(),
                            form_data.clone(),
                        );
                    }
                    RequestBodyContent::EMPTY => {
                    }
                }
            }
        }

        std::task::Poll::Ready(Ok(()))
    }
}
