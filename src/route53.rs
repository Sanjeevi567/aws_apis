use aws_config::SdkConfig;
use aws_sdk_route53::Client as Route53Client;
use colored::Colorize;

pub struct Route53Ops<'a> {
    config: &'a SdkConfig,
}
impl<'a> Route53Ops<'a> {
    pub fn build(config: &'a SdkConfig) -> Self {
        Self { config }
    }
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }
    pub async fn create_hosted_zone(&self, domain_name: &str, caller_reference: &str) {
        let config = self.get_config();
        let client = Route53Client::new(config);
        client
            .create_hosted_zone()
            .name(domain_name)
            .caller_reference(caller_reference)
            .send()
            .await
            .expect("Error while creating domain");
        println!(
            "The domain name {} has been registered sucessfully",
            domain_name.green().bold()
        );
    }
}
