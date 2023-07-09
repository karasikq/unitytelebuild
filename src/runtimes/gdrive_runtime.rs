use google_drive3::api::File;
use google_drive3::{hyper, hyper_rustls, DriveHub, Error};
use std::path::PathBuf;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

pub struct HubWrapper {
    hub: DriveHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl HubWrapper {
    pub async fn new() -> Self {
        let app_secret = yup_oauth2::read_application_secret("client_secrets.json")
            .await
            .expect("clientsecret.json");

        let auth = InstalledFlowAuthenticator::builder(
            app_secret,
            InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await
        .unwrap();
        let scopes = &["https://www.googleapis.com/auth/drive.metadata.readonly"];

        match auth.token(scopes).await {
            Err(e) => println!("OAUT2 error: {:?}", e),
            Ok(t) => println!("The token is {:?}", t),
        }

        let hub = DriveHub::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            auth,
        );
        HubWrapper { hub }
    }

    pub async fn upload_file(&self, path: PathBuf) {
        let result = self
            .hub
            .files()
            .create(File::default())
            .param("fields", "files(id, name, webContentLink, webViewLink)")
            .upload(
                std::fs::File::open(path).unwrap(),
                "application/octet-stream".parse().unwrap(),
            )
            .await;
        match result {
            Err(_) => todo!(),
            Ok((_body, file)) => {
                println!(
                    "File: {}",
                    file.name.unwrap_or_else(|| String::from("Unnamed"))
                );
                if let Some(download_url) = file.web_content_link {
                    println!("Download link: {}", download_url);
                } else if let Some(web_view_link) = file.web_view_link {
                    println!("Web view link: {}", web_view_link);
                }
            }
        };
    }

    pub async fn print_file_links(&self) {
        let down_res = self
            .hub
            .files()
            .list()
            .q("'root' in parents")
            .param("fields", "files(id, name, webContentLink, webViewLink)")
            .doit()
            .await;

        match down_res {
            Err(e) => match e {
                Error::HttpError(_)
                | Error::Io(_)
                | Error::MissingAPIKey
                | Error::MissingToken(_)
                | Error::Cancelled
                | Error::UploadSizeLimitExceeded(_, _)
                | Error::Failure(_)
                | Error::BadRequest(_)
                | Error::FieldClash(_)
                | Error::JsonDecodeError(_, _) => println!("{}", e),
            },
            Ok(res) => {
                res.1.files.unwrap().into_iter().for_each(|file| {
                    println!(
                        "File: {}",
                        file.name.unwrap_or_else(|| String::from("Unnamed"))
                    );
                    if let Some(download_url) = file.web_content_link {
                        println!("Download link: {}", download_url);
                    } else if let Some(web_view_link) = file.web_view_link {
                        println!("Web view link: {}", web_view_link);
                    }
                });
            }
        }
    }
}
