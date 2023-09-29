use aws_config::SdkConfig;
use aws_sdk_pinpoint::{
    types::{CreateApplicationRequest, EmailTemplateRequest, NumberValidateRequest},
    Client as PinPointClient,
};

pub struct PinPointOps{
    config: SdkConfig,
}
impl PinPointOps {
    pub fn build(config: SdkConfig) -> Self {
        Self { config }
    }
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }
    pub async fn create_app(&self, app_name: &str) -> ApplicationResponse {
        let config = self.get_config();
        let client = PinPointClient::new(config);

        let create_request_builder = CreateApplicationRequest::builder().name(app_name).build();

        let output = client
            .create_app()
            .create_application_request(create_request_builder)
            .send()
            .await
            .expect("Error while creating app in pinpoint\n");
        let response = output.application_response;
        let mut application_response = ApplicationResponse::default();

        if let Some(applicationresp) = response {
            let arn = applicationresp.arn;
            let id = applicationresp.id;
            let name = applicationresp.name;
            let creation_date = applicationresp.creation_date;
            application_response =
                ApplicationResponse::build_application_response(arn, id, name, creation_date)
        }
        application_response
    }

    pub async fn create_email_template(
        &self,
        template_name: &str,
        template_data: &str,
        default_values: &str,
        subject: &str,
        template_description: &str,
    ) -> (Option<String>, Option<String>) {
        let config = self.get_config();
        let client = PinPointClient::new(config);

        let email_template_builder = EmailTemplateRequest::builder()
            .default_substitutions(default_values)
            .html_part(template_data)
            .subject(subject)
            .template_description(template_description)
            .build();
        let output = client
            .create_email_template()
            .template_name(template_name)
            .email_template_request(email_template_builder)
            .send()
            .await
            .expect("Error while creating Email Template\n");
        let msg_body = output.create_template_message_body;
        let mut message: Option<String> = None;
        let mut arn: Option<String> = None;
        if let Some(msg_body_) = msg_body {
            message = msg_body_.message;
            arn = msg_body_.arn;
        }
        (message, arn)
    }
    pub async fn phone_number_validate(&self, phone_number: &str, iso_code: &str) {
        let config = self.get_config();
        let client = PinPointClient::new(config);

        let phone_number_builder = NumberValidateRequest::builder()
            .phone_number(phone_number)
            .iso_country_code(iso_code)
            .build();

        client
            .phone_number_validate()
            .number_validate_request(phone_number_builder)
            .send()
            .await
            .expect("Error while validating PhoneNumber");
    }
}

#[derive(Default)]
pub struct ApplicationResponse {
    pub arn: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub creation_date: Option<String>,
}
impl ApplicationResponse {
    fn build_application_response(
        arn: Option<String>,
        id: Option<String>,
        name: Option<String>,
        creation_date: Option<String>,
    ) -> Self {
        Self {
            arn,
            id,
            name,
            creation_date,
        }
    }
    pub fn get_arn(&self) -> Option<&str> {
        let arn = if let Some(arn_) = self.arn.as_deref() {
            Some(arn_)
        } else {
            None
        };
        arn
    }
    pub fn get_app_name(&self) -> Option<&str> {
        let app_name = if let Some(name) = self.name.as_deref() {
            Some(name)
        } else {
            None
        };
        app_name
    }
    pub fn get_creation_date(&self) -> Option<&str> {
        let date = if let Some(creation) = self.creation_date.as_deref() {
            Some(creation)
        } else {
            None
        };
        date
    }
    pub fn get_id(&self) -> Option<&str> {
        let id = if let Some(id_) = self.id.as_deref() {
            Some(id_)
        } else {
            None
        };
        id
    }
}
