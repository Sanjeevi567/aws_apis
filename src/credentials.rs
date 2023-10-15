pub use aws_config::{load_from_env, SdkConfig};
use aws_credential_types::{
    provider::{ProvideCredentials, SharedCredentialsProvider},
    Credentials,
};
use aws_types::region::Region;
use colored::Colorize;
use dotenv::dotenv;
use std::env::var;

#[derive(Debug, Default)]
pub struct CredentInitialize {
    access_id: Option<String>,
    secret_key: Option<String>,
    region: Option<String>,
    provider_name: Option<String>,
}
///You can initialize the credentials inside the 'update' function if you don't want to configure
/// them later.
impl CredentInitialize {
    pub fn update(&mut self, access_id: &str, secret_key: &str, region: Option<&str>) {
        self.access_id = Some(access_id.into());
        self.secret_key = Some(secret_key.into());
        self.region = match region {
            Some(region) => Some(region.to_string()),
            None => Some(self.get_region_name()),
        };
        self.provider_name = Some("aws".into());
    }

    pub fn credential(&mut self) -> SdkConfig {
        let access_id = self.access_id.clone().take().unwrap_or_default();

        let secret_key = self.secret_key.clone().take().unwrap_or_default();

        let credential = Credentials::new(
            access_id.to_owned(),
            secret_key.to_owned(),
            None,
            None,
            "aws",
        );
        let shared = SharedCredentialsProvider::new(credential);
        let region = self.region.clone().take().unwrap_or_default();
        let region = Region::new(region);
        SdkConfig::builder()
            .credentials_provider(shared)
            .region(Some(region))
            .build()
    }
    pub fn build(&mut self) -> SdkConfig {
        self.credential()
    }
    pub fn print_credentials(&self) {
        match (self.access_id.as_deref(), self.secret_key.as_deref()) {
            (Some(access_id), Some(secret_key)) => {
                println!("Access Key Id: {}", access_id.green().bold());
                println!("Secret AccessKey Id: {}", secret_key.green().bold());
            }
            _ => {}
        }
        if let Some(region) = self.region.as_deref() {
            println!("Region: {}\n", region.green().bold());
        }
    }
    pub fn get_region_name(&self) -> String {
        dotenv().ok();
        var("REGION").unwrap_or("The region value is read from the .env file in the current directory if it is not provided in the credential file.".into())
    }
    pub fn empty(&mut self) {
        self.access_id = None;
        self.secret_key = None;
        self.region = None;
        self.provider_name = None;
    }
}

/// Returns the [`Credentials`](https://docs.rs/aws-credential-types/0.56.1/aws_credential_types/struct.Credentials.html?search=sdkconfig#) types to retrieve access_id and secret_key, as well as
/// the region name, from the configuration.
pub async fn load_credential_from_env() -> (Credentials, Option<String>) {
    println!("{}\n",r#"The configuration path is "$HOME/.aws/credentials" on Linux and macOS, and "%USER_PROFILE%/.aws/credentials" on Windows"#.green().bold());

    println!("Attempting to retrieve credentials from the configuration file\n");

    let config = aws_config::load_from_env().await;
    let shared_credential = config.credentials_provider().unwrap();
    let credentials = shared_credential.provide_credentials().await.unwrap();
    println!(
    "{}\n",
    "The region value is read from the .env file in the current directory if it is not provided in the credential file".blue().bold());
    let region = match config.region() {
        Some(region) => Some(region.to_string()),
        None => None,
    };

    (credentials, region)
}
