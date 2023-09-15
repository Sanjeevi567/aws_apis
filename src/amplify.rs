use std::fs::OpenOptions;

use aws_config::SdkConfig;
use aws_sdk_amplify::{
    types::{App, Platform},
    Client as AmplifyClient,
};

use std::io::Write;
use std::ops::Deref;
pub struct AmplifyOps {
    config: SdkConfig,
}
impl AmplifyOps {
    pub fn build(config: SdkConfig) -> Self {
        Self { config }
    }
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }
    pub async fn create_app(
        &self,
        project_description: Option<String>,
        project_name: &str,
        source: Option<String>,
        platform_type: Option<Platform>,
    ) {
        let config = self.get_config();
        let client = AmplifyClient::new(config);

        let create_app = client
            .create_app()
            .name(project_name)
            .set_repository(source)
            .set_platform(platform_type)
            .set_description(project_description)
            .send()
            .await
            .expect("Error while creating App\n");
        if let Some(app) = create_app.app {
            let app_id = app.app_id;
            if let Some(app_id_) = app_id {
                let application_id = format!(
                    "The application identifier for the app named '{project_name}' is:  {app_id_}"
                );
                let path_name = format!("{project_name}_app_id.txt");
                let mut file = OpenOptions::new()
                    .create(true)
                    .read(true)
                    .write(true)
                    .open(&path_name)
                    .expect("Error while creating file\n");
                match file.write_all(application_id.as_bytes()) {
                    Ok(_) => println!("The application ID for the app named {project_name} has been written to the current directory\n"),
                    Err(_) => println!("Error writing app id\n"),
                }
            }
        }
    }
    pub async fn get_app(&self, app_id: &str) -> Option<aws_sdk_amplify::types::App> {
        let config = self.get_config();
        let client = AmplifyClient::new(config);
        let get_app = client
            .get_app()
            .app_id(app_id)
            .send()
            .await
            .expect("Error while getting App Info\n");
        get_app.app
    }
}
pub struct AppInfo(App);
impl Deref for AppInfo {
    type Target = App;
    fn deref(&self) -> &App {
        &self.0
    }
}
