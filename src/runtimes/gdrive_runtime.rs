use std::path::Path;

use google_drive3::api::File;
use google_drive3::{hyper, hyper_rustls, DriveHub};
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

pub struct HubWrapper {
    hub: DriveHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl HubWrapper {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
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
        Ok(HubWrapper { hub })
    }

    pub async fn upload_file(&self, path: String) -> Result<File, Box<dyn std::error::Error + Send + Sync>> {
        let file = File{
            name: Some(Path::new(&path).file_name().unwrap().to_os_string().into_string().unwrap()),
            .. File::default()
        };
        let result = self
            .hub
            .files()
            .create(file)
            .param("fields", "id, name, webContentLink, webViewLink")
            .upload(
                std::fs::File::open(path).unwrap(),
                "application/octet-stream".parse().unwrap(),
            )
            .await;
        match result {
            Err(e) => {
                Err(Box::new(e))
            },
            Ok((_body, file)) => {
                Ok(file)
            }
        }
    }
}
