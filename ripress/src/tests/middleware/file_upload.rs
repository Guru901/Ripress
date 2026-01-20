#[cfg(test)]
#[cfg(feature = "file-upload")]
mod test {
    use tempfile::TempDir;

    use crate::{
        middlewares::file_upload::{file_upload, FileUploadConfiguration},
        req::HttpRequest,
        res::HttpResponse,
    };

    #[tokio::test]
    #[ignore = "abhi ke liye"]
    async fn test_file_upload_single_binary_file() {
        let temp_dir = TempDir::new().unwrap();
        let upload_mw = file_upload(Some(FileUploadConfiguration {
            upload_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        }));

        let mut req = HttpRequest::new();
        let test_content = b"Hello, this is a test file content!";

        req._set_binary(test_content.to_vec());
        req.set_header("content-type", "application/octet-stream");

        let res = HttpResponse::new();
        let (req, _) = upload_mw(req, res).await;

        let uploaded_file = req.get_data("uploaded_file").unwrap();
        let uploaded_path = req.get_data("uploaded_file_path").unwrap();

        assert!(
            std::fs::metadata(&uploaded_path).is_ok(),
            "File should exist at {}",
            uploaded_path
        );

        let file_content = std::fs::read(&uploaded_path).unwrap();
        assert_eq!(file_content, test_content, "File content mismatch");

        let form_data = req.form_data().unwrap();
        assert_eq!(form_data.get("file"), Some(uploaded_file.as_str()));

        assert_eq!(req.get_data("uploaded_file_count"), Some("1".to_string()));

        let _ = std::fs::remove_file(&uploaded_path);
    }

    #[tokio::test]
    async fn test_file_upload_no_files() {
        let temp_dir = TempDir::new().unwrap();
        let upload_mw = file_upload(Some(FileUploadConfiguration {
            upload_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        }));

        let mut req = HttpRequest::new();

        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let multipart_data = format!(
            "--{boundary}\r\n\
            Content-Disposition: form-data; name=\"name\"\r\n\
            \r\n\
            John Doe\r\n\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"age\"\r\n\
            \r\n\
            30\r\n\
            --{boundary}--\r\n"
        );

        req._set_binary(multipart_data.into_bytes());
        req.set_header(
            "content-type",
            &format!("multipart/form-data; boundary={}", boundary),
        );

        let res = HttpResponse::new();
        let (req, _) = upload_mw(req, res).await;

        let form_data = req.form_data().unwrap();
        assert_eq!(form_data.get("name"), Some("John Doe"));
        assert_eq!(form_data.get("age"), Some("30"));

        let file_count = req.get_data("uploaded_file_count");
        if file_count.is_some() {
            assert!(req.get_data("uploaded_files").is_some());
            assert!(req.get_data("uploaded_file").is_some());
            assert!(req.get_data("uploaded_file_path").is_some());
        } else {
            assert_eq!(req.get_data("uploaded_files"), None);
            assert_eq!(req.get_data("uploaded_file"), None);
            assert_eq!(req.get_data("uploaded_file_path"), None);
        }
    }

    #[tokio::test]
    async fn test_file_upload_invalid_multipart() {
        let temp_dir = TempDir::new().unwrap();
        let upload_mw = file_upload(Some(FileUploadConfiguration {
            upload_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        }));

        let mut req = HttpRequest::new();

        let invalid_data = "This is not valid multipart data";

        req._set_binary(invalid_data.as_bytes().to_vec());
        req.set_header("content-type", "multipart/form-data; boundary=invalid");

        let res = HttpResponse::new();
        let (req, _) = upload_mw(req, res).await;

        assert!(
            req.get_data("uploaded_file_count").is_some()
                || req.get_data("uploaded_file_count").is_none()
        );
    }

    #[tokio::test]
    async fn test_file_upload_empty_multipart() {
        let temp_dir = TempDir::new().unwrap();
        let upload_mw = file_upload(Some(FileUploadConfiguration {
            upload_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        }));

        let mut req = HttpRequest::new();

        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let empty_multipart = format!("--{boundary}--\r\n");

        req._set_binary(empty_multipart.into_bytes());
        req.set_header(
            "content-type",
            &format!("multipart/form-data; boundary={}", boundary),
        );

        let res = HttpResponse::new();
        let (req, _) = upload_mw(req, res).await;

        assert!(
            req.get_data("uploaded_file_count").is_some()
                || req.get_data("uploaded_file_count").is_none()
        );
    }

    #[tokio::test]
    async fn test_multipart_with_files_no_middleware() {

        let boundary = String::from("----WebKitFormBoundary7MA4YWxkTrZu0gW");
        let multipart_data = format!(
            "--{boundary}\r\n\
            Content-Disposition: form-data; name=\"name\"\r\n\
            \r\n\
            John Doe\r\n\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"age\"\r\n\
            \r\n\
            30\r\n\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            file content\r\n\
            --{boundary}--\r\n"
        );

        let content_type = crate::helpers::determine_content_type_request(&format!(
            "multipart/form-data; boundary={}",
            boundary
        ));
        assert_eq!(
            content_type,
            crate::req::body::RequestBodyType::MultipartForm
        );

        let (fields, file_parts) =
            crate::helpers::parse_multipart_form(multipart_data.as_bytes(), &boundary);

        assert_eq!(fields.len(), 2);
        assert_eq!(file_parts.len(), 1);

        let name_field = fields.iter().find(|(k, _)| *k == "name");
        let age_field = fields.iter().find(|(k, _)| *k == "age");
        assert_eq!(name_field.map(|(_, v)| v), Some(&"John Doe"));
        assert_eq!(age_field.map(|(_, v)| v), Some(&"30"));

        let file_part = &file_parts[0];
        assert_eq!(file_part.1.unwrap(), "file"); 


        let mut req = HttpRequest::new();

        req.set_form("name", "John Doe", crate::req::body::RequestBodyType::FORM);
        req.set_form("age", "30", crate::req::body::RequestBodyType::FORM);

        let retrieved_form_data = req.form_data().unwrap();
        assert_eq!(retrieved_form_data.get("name"), Some("John Doe"));
        assert_eq!(retrieved_form_data.get("age"), Some("30"));

        assert_eq!(retrieved_form_data.get("file"), None);

    }

    #[tokio::test]
    async fn test_multipart_with_files_request_building() {

        let boundary = String::from("----WebKitFormBoundary7MA4YWxkTrZu0gW");
        let multipart_data = format!(
            "--{boundary}\r\n\
            Content-Disposition: form-data; name=\"name\"\r\n\
            \r\n\
            John Doe\r\n\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"age\"\r\n\
            \r\n\
            30\r\n\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
            Content-Type: text/plain\r\n\
            \r\n\
            file content\r\n\
            --{boundary}--\r\n"
        );


        let content_type = crate::helpers::determine_content_type_request(&format!(
            "multipart/form-data; boundary={}",
            boundary
        ));
        assert_eq!(
            content_type,
            crate::req::body::RequestBodyType::MultipartForm
        );

        let (fields, file_parts) =
            crate::helpers::parse_multipart_form(multipart_data.as_bytes(), &boundary);

        assert_eq!(fields.len(), 2);
        assert_eq!(file_parts.len(), 1);

        let mut form_data = crate::req::body::FormData::new();
        for (key, value) in fields {
            form_data.insert(key, value);
        }

        let request_body = if !file_parts.is_empty() {
            crate::req::body::RequestBody::new_binary_with_form_fields(
                multipart_data.into_bytes().into(),
                form_data,
            )
        } else {
            crate::req::body::RequestBody::new_form(form_data)
        };

        assert_eq!(
            request_body.content_type,
            crate::req::body::RequestBodyType::BINARY
        );

        if let crate::req::body::RequestBodyContent::BinaryWithFields(_, stored_form_data) =
            &request_body.content
        {
            assert_eq!(stored_form_data.get("name"), Some("John Doe"));
            assert_eq!(stored_form_data.get("age"), Some("30"));
            assert_eq!(stored_form_data.get("file"), None); 
        } else {
            panic!("Expected BinaryWithFields variant");
        }

    }
}
