use std::{fs::OpenOptions, io::Write};

use aws_config::SdkConfig;
use aws_sdk_sns::Client as SnsClient;
use colored::Colorize;

pub struct SnsOps<'a> {
    config: &'a SdkConfig,
}
impl<'a> SnsOps<'a> {
    pub fn build(config: &'a SdkConfig) -> Self {
        Self { config }
    }
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }
    pub async fn create_sandbox_phone_number(&self, phone_number: &str) {
        let config = self.get_config();
        let client = SnsClient::new(config);

        client
            .create_sms_sandbox_phone_number()
            .phone_number(phone_number)
            .send()
            .await
            .expect("Error while sending Message\n");
        println!("{}\n", "Phone Number has been added".green().bold());
    }
    pub async fn list_sms_sandbox_numbers(&self) -> String {
        let config = self.get_config();
        let client = SnsClient::new(config);

        let output = client
            .list_sms_sandbox_phone_numbers()
            .send()
            .await
            .expect("Error while listing sms sanbox numbers\n");
        let mut phonenumber_with_status = Vec::new();
        if let Some(phone_numbers_info) = output.phone_numbers {
            phone_numbers_info.into_iter().for_each(|info| {
                if let (Some(phone_number), Some(status)) = (info.phone_number, info.status) {
                    let mut number_with_status = phone_number;
                    number_with_status.push_str("    ");
                    number_with_status.push_str(status.as_str());
                    phonenumber_with_status.push(number_with_status);
                }
            });
        }
        phonenumber_with_status.join("\n")
    }
    pub async fn verify_phone_number(&self, phone_number: &str, otp: &str) {
        let config = self.get_config();
        let client = SnsClient::new(config);

        client
            .verify_sms_sandbox_phone_number()
            .phone_number(phone_number)
            .one_time_password(otp)
            .send()
            .await
            .expect("Error while verifying Phone Number");
        println!("{}\n", "SMS has been verified successfully".green().bold());
    }
    pub async fn create_topic(&self, topic_name: &str) {
        let config = self.get_config();
        let client = SnsClient::new(config);

        let output = client
            .create_topic()
            .name(topic_name)
            .send()
            .await
            .expect("Error while creating topic\n");
        println!("{}\n", "The topic was created successfully".green().bold());
        if let Some(output_) = output.topic_arn {
            let arn = output_.green().bold();
            println!("The Amazon Resource Name (ARN) for the SNS topic is: {arn}\n");
            let mut file = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open("sns_topic_arn.txt")
                .expect("Error while creating file");
            let buf = format!("The Amazon Resource Name (ARN) for the SNS topic is: {arn}\n");
            match file.write_all(buf.as_bytes()) {
                Ok(_) => println!(
                    "{}\n",
                    "The ARN has been written to the current directory."
                        .green()
                        .bold()
                ),
                Err(_) => println!("Error while writing data"),
            };
        }
    }
    pub async fn subscription(&self, topic_arn: &str, protocol: &str, phone_number: &str) {
        let config = self.get_config();
        let client = SnsClient::new(config);

        let output = client
            .subscribe()
            .topic_arn(topic_arn)
            .endpoint(phone_number)
            .protocol(protocol)
            .send()
            .await
            .expect("Error while subscribing\n");
        if let Some(subscription_arn) = output.subscription_arn {
            let colored_arn = subscription_arn.green().bold();
            println!("Subscription ARN: {colored_arn}\n");
            let mut file = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open("sns_topic_arn.txt")
                .expect("Error while creating file");
            let buf =
                format!("The Amazon Resource Name for the Subscription is: {subscription_arn}\n");
            match file.write_all(buf.as_bytes()) {
                Ok(_) => println!(
                    "{}\n",
                    "The ARN is written to the current directory."
                        .green()
                        .bold()
                ),
                Err(_) => println!("Error while writing data"),
            }
        }
    }
    pub async fn publish(&self, message: &str, topic_arn: &str) {
        let config = self.get_config();
        let client = SnsClient::new(config);

        client
            .publish()
            .topic_arn(topic_arn)
            .message(message)
            .subject("Testing")
            .send()
            .await
            .expect("Error while sending messages to topic");
        println!(
            "{}\n",
            "Messages have been sent successfully....".green().bold()
        );
    }
}
