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
        // Get a mutable reference to self
        let this = self.get_mut();

        // Convert the buffer to bytes
        let new_bytes = bytes::Bytes::copy_from_slice(buf);

        // Append the new bytes to the existing body content
        match &mut this.body.content {
            RequestBodyContent::BINARY(existing_bytes) => {
                // For binary content, we need to concatenate the bytes
                // Since Bytes doesn't support direct concatenation, we'll convert to Vec and back
                let mut combined = existing_bytes.to_vec();
                combined.extend_from_slice(buf);
                this.body.content = RequestBodyContent::BINARY(combined.into());
            }
            RequestBodyContent::BinaryWithFields(existing_bytes, form_data) => {
                // For binary with fields, append to the binary part
                let mut combined = existing_bytes.to_vec();
                combined.extend_from_slice(buf);
                this.body.content =
                    RequestBodyContent::BinaryWithFields(combined.into(), form_data.clone());
            }
            RequestBodyContent::TEXT(text_data) => {
                // For text content, append the bytes as UTF-8 string
                if let Ok(new_text) = String::from_utf8(buf.to_vec()) {
                    // Use as_str_lossy() to handle potential UTF-8 errors gracefully
                    let existing_text = text_data.as_str_lossy();
                    let combined_text = format!("{}{}", existing_text, new_text);
                    this.body.content = RequestBodyContent::TEXT(TextData::new(combined_text));
                } else {
                    // If the new bytes aren't valid UTF-8, convert to binary
                    let mut combined = text_data.as_bytes().to_vec();
                    combined.extend_from_slice(buf);
                    this.body.content = RequestBodyContent::BINARY(combined.into());
                    this.body.content_type = RequestBodyType::BINARY;
                }
            }
            RequestBodyContent::JSON(json_value) => {
                // For JSON content, append as text (this might not be valid JSON)
                let json_str = json_value.to_string();
                let mut combined = json_str.as_bytes().to_vec();
                combined.extend_from_slice(buf);
                // Convert to text since the result might not be valid JSON
                if let Ok(combined_text) = String::from_utf8(combined.clone()) {
                    this.body.content = RequestBodyContent::TEXT(TextData::new(combined_text));
                    this.body.content_type = RequestBodyType::TEXT;
                } else {
                    this.body.content = RequestBodyContent::BINARY(combined.into());
                    this.body.content_type = RequestBodyType::BINARY;
                }
            }
            RequestBodyContent::FORM(form_data) => {
                // For form data, append as text
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
                // For empty content, start with the new bytes
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
        // No buffering to flush, so we're always ready
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        // No special shutdown needed for request body
        std::task::Poll::Ready(Ok(()))
    }
}

impl AsyncRead for HttpRequest {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // Get a mutable reference to self
        let this = unsafe { self.get_unchecked_mut() };

        // Convert the request body content to bytes
        let body_bytes = match &this.body.content {
            RequestBodyContent::TEXT(text_data) => text_data.as_bytes().to_vec(),
            RequestBodyContent::JSON(json_value) => {
                serde_json::to_vec(json_value).unwrap_or_default()
            }
            RequestBodyContent::FORM(form_data) => form_data.to_string().as_bytes().to_vec(),
            RequestBodyContent::BINARY(bytes) => bytes.to_vec(),
            RequestBodyContent::BinaryWithFields(bytes, _form_data) => bytes.to_vec(),
            RequestBodyContent::EMPTY => Vec::new(),
        };

        // If we have data to read
        if !body_bytes.is_empty() {
            let bytes_to_copy = std::cmp::min(buf.remaining(), body_bytes.len());
            let start_pos = 0;
            let end_pos = bytes_to_copy;

            // Copy bytes to the buffer
            buf.put_slice(&body_bytes[start_pos..end_pos]);

            // Remove the bytes we just read from the body
            if bytes_to_copy == body_bytes.len() {
                // If we read all bytes, set to empty and sync content_type
                this.body.content = RequestBodyContent::EMPTY;
                this.body.content_type = RequestBodyType::EMPTY;
            } else {
                // If we read partial bytes, update the body with remaining bytes
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
                        // For JSON, convert remaining bytes to text or binary
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
                        // For form data, convert remaining bytes to text or binary
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
                        // Should not happen, but handle gracefully
                    }
                }
            }
        }

        // Always return Ready since we're reading from memory
        std::task::Poll::Ready(Ok(()))
    }
}
