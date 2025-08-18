use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};
use tokio::fs::{File, create_dir_all};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub fn file_upload(
    upload_dir: Option<&str>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + Clone + 'static {
    let upload_path = upload_dir.unwrap_or("uploads").to_string();

    move |mut req, res| {
        let upload_path = upload_path.clone();
        Box::pin(async move {
            if let Ok(bytes) = req.bytes() {
                // Ensure the upload directory exists
                if let Err(e) = create_dir_all(&upload_path).await {
                    eprintln!("Failed to create upload directory '{}': {}", upload_path, e);
                    return (req, Some(res));
                }

                let extension = infer::get(&bytes)
                    .and_then(|info| {
                        let ext = info.extension();
                        Some(ext)
                    })
                    .unwrap_or("bin");

                let id = Uuid::new_v4();

                let filename = format!("{}.{}", id, extension);
                let filename_with_path = format!("{}/{}.{}", upload_path, id, extension);

                match File::create(&filename_with_path).await {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(bytes).await {
                            eprintln!(
                                "Failed to write file '{}': {}",
                                filename_with_path.display(),
                                e
                            );
                            // TODO: set an explicit error status (e.g., 500) on `res` before returning.
                            return (req, Some(res));
                        }
                        req.set_data("uploaded_file", filename.as_str());
                        let file_path_str = filename_with_path.to_string_lossy().into_owned();
                        req.set_data("uploaded_file_path", file_path_str);
                        (req, None)
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to create file '{}': {}",
                            filename_with_path.display(),
                            e
                        );
                        // TODO: set an explicit error status (e.g., 500) on `res` before returning.
                        (req, Some(res))
                    }
                }
            } else {
                (req, Some(res))
            }
        })
    }
}
