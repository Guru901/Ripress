use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};
use tokio::fs::{File, create_dir_all};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub fn file_upload(
    upload_dir: Option<&str>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + Clone + 'static {
    let upload_path = upload_dir.unwrap_or("uploads").to_string();

    move |mut req, _res| {
        let upload_path = upload_path.clone();
        Box::pin(async move {
            // Try to upload file if binary content is available
            match req.bytes() {
                Ok(bytes) => {
                    // Ensure the upload directory exists
                    if let Err(e) = create_dir_all(&upload_path).await {
                        eprintln!("Failed to create upload directory '{}': {}", upload_path, e);
                        // Continue without file upload - don't short-circuit the request
                        return (req, None);
                    }

                    let extension = infer::get(bytes)
                        .map(|info| info.extension())
                        .unwrap_or("bin");

                    let id = Uuid::new_v4();

                    let filename = format!("{}.{}", id, extension);
                    let filename_with_path = format!("{}/{}.{}", upload_path, id, extension);

                    match File::create(&filename_with_path).await {
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(bytes).await {
                                eprintln!("Failed to write file '{}': {}", filename_with_path, e);
                                // Continue without file upload - don't short-circuit the request
                                return (req, None);
                            }
                            // File upload successful - set the data and continue
                            req.set_data("uploaded_file", filename.as_str());
                            req.set_data("uploaded_file_path", filename_with_path.as_str());
                            (req, None)
                        }
                        Err(e) => {
                            eprintln!("Failed to create file '{}': {}", filename_with_path, e);
                            // Continue without file upload - don't short-circuit the request
                            (req, None)
                        }
                    }
                }
                Err(error_msg) => {
                    // Log the error for debugging but don't fail the request
                    eprintln!("File upload middleware: {}", error_msg);

                    // For non-binary requests, just continue without file upload
                    // This allows the request to proceed normally
                    (req, None)
                }
            }
        })
    }
}
