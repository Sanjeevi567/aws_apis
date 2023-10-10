use crate::{create_email_identities_pdf, create_email_pdf};

use self::SimpleOrTemplate::{Simple_, Template_};
use aws_config::{imds::client, SdkConfig};
use aws_sdk_sesv2 as sesv2;
use colored::Colorize;
use dotenv::dotenv;
use regex::Regex;
use sesv2::{
    operation::send_email::builders::SendEmailFluentBuilder,
    types::{Body, Content, Destination, EmailContent, EmailTemplateContent, Message, Template},
    Client as SesClient,
};
use std::{
    env::var,
    fs::{self, File, OpenOptions},
    io::Write,
};

/// The core structure for performing operations on [`SESv2`](https://docs.rs/aws-sdk-sesv2/latest/aws_sdk_sesv2/struct.Client.html) (Simple Email Service Version 2)
/// clients eliminates the need for users of the API to provide credentials each
/// time they use the service. Instead, these credentials are abstracted by this
/// structure along with its inherent functions and methods.
#[derive(Debug)]
pub struct SesOps {
    config: SdkConfig,
}

impl SesOps {
    ///When calling this function, it builds the credentials and the SesOps struct.
    pub fn build(config: SdkConfig) -> Self {
        Self { config: config }
    }

    /// These are not retrieved from an AWS service. In other words, these values act as proxies for the actual data if you're familiar with these details

    /// The 'from' address has to be verified since this is the base email used to send mail to others
    pub fn get_from_address(&self) -> String {
        dotenv().ok();
        var("FROM_ADDRESS").unwrap_or("It appears that you haven't set the 'FROM_ADDRESS' environment variable. You can only skip this input if you have configured the variable".into())
    }
    /// The template name must correspond to the credentials you used, and the
    /// template data must accurately match the template employed by those services
    pub fn get_template_name(&self) -> String {
        dotenv().ok();
        var("TEMPLATE_NAME").unwrap_or("It appears that you haven't set the 'TEMPLATE_NAME' environment variable. You can only skip this input if you have configured the variable".into())
    }
    /// If the list name does not exist, i.e., if it has not been set using the
    /// appropriate methods, an error will occur when attempting to use it.
    pub fn get_list_name(&self) -> String {
        dotenv().ok();
        var("LIST_NAME").unwrap_or("It appears that you haven't set the 'LIST_NAME' environment variable. You can only skip this input if you have configured the variable".into())
    }

    /// This is a private function used internally to verify service credentials.
    /// By doing so, users of the API are spared from having to repeatedly specify
    /// their credentials whenever they use the service
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }
    /// These operations are asynchronous functions, so be sure to await them;
    /// otherwise, no computation will occur at all
    pub async fn create_contact_list_name(&self, list_name: &str, description: Option<String>) {
        let config = self.get_config();
        let client = SesClient::new(config);
        let available_list_names = self
            .list_contact_lists()
            .await
            .into_iter()
            .map(|to_string| {
                let mut add_space = to_string;
                add_space.push(' ');
                add_space
            })
            .collect::<String>();
        if available_list_names.is_empty() {
            let client = client
                .create_contact_list()
                .contact_list_name(list_name)
                .set_description(description);
            let colored_error = "Error from create_contact_list_name()".red().bold();
            client
                .send()
                .await
                .map(|_| {
                    let colored_list = list_name.green().bold();
                    println!("The list named {colored_list} created sucessfully\n")
                })
                .expect(&colored_error);
        } else {
            println!(
                "{}\n",
                "Possible reasons this operation may have failed are"
                    .red()
                    .bold()
            );
            println!("1) You may not have the proper credentials or region to execute this operation.\nYou need the following permissions: '{}' and '{}'","ses:CreateContactList".yellow().bold(),"ses:ListContactLists".yellow().bold());
            println!("{}\n","2) Only one contact list name per account or region can be created.\nHere is the contact list name in your account and region, if there is any".yellow().bold());
            let lists = self.list_contact_lists().await;
            for list in lists {
                println!("    {}", list.green().bold());
            }
            println!("");
            println!(
                "{}",
                "If you see anything, you must delete it before creating a new contact list name"
                    .yellow()
                    .bold()
            );
            println!("{}\n","Please note that deleting a contact list name will also delete all the emails in that list".red().bold());
        }
    }
    pub async fn list_contact_lists(&self) -> Vec<String> {
        let config: &SdkConfig = self.get_config();
        let client = SesClient::new(config);
        let outputs = client
            .list_contact_lists()
            .send()
            .await
            .expect("Error while listing contact lists\n");
        let mut list_names = Vec::new();
        if let Some(lists) = outputs.contact_lists {
            lists.into_iter().for_each(|contact_list| {
                if let Some(name) = contact_list.contact_list_name {
                    list_names.push(name);
                }
            })
        }
        list_names
    }
    pub async fn list_email_identity(&self) -> Vec<String> {
        let config = self.get_config();
        let client = SesClient::new(config);
        let outputs = client
            .list_email_identities()
            .send()
            .await
            .expect("Error while listing email Identities\n");
        let mut vec_of_identity_info = Vec::new();
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("EmailIdentyDetails.txt")
            .expect("Error while creating file");
        if let Some(identity_info) = outputs.email_identities {
            identity_info.into_iter().for_each(|info| {
                if let Some(identity_type) = info.identity_type {
                    let type_ = identity_type.as_str().to_string();
                    let buf = format!("Identity Type: {type_}");
                    file.write_all(buf.as_bytes()).unwrap();
                    vec_of_identity_info.push(type_);
                }
                if let Some(identity_name) = info.identity_name {
                    let buf = format!("Identity Name: {identity_name}");
                    file.write_all(buf.as_bytes()).unwrap();
                    vec_of_identity_info.push(identity_name);
                }
                let sending_enabled = format!("{}", info.sending_enabled);
                let buf = format!("Is Sending Enabled: {sending_enabled}");
                file.write_all(buf.as_bytes()).unwrap();
                vec_of_identity_info.push(sending_enabled);
                if let Some(status) = info.verification_status {
                    let status = status.as_str().to_string();
                    let buf = format!("Verification Status: {status}\n\n");
                    file.write_all(buf.as_bytes()).unwrap();
                    vec_of_identity_info.push(status);
                }
            });
            match File::open("EmailIdentyDetails.txt") {
                Ok(_) => println!("Email Identity Details are written to a file called '{}'\n To view the emails, please check the current directory","EmailIdentyDetails.txt".green().bold()),
                Err(_) => println!("{}\n", "Error while writing file".red().bold()),
            }
        }
        vec_of_identity_info
    }

    pub async fn delete_contact_list_name(&self, contact_list_name: &str) {
        let config = self.get_config();
        let client = SesClient::new(config);
        if self.is_contact_list_name_exist(contact_list_name).await {
            client
                .delete_contact_list()
                .contact_list_name(contact_list_name)
                .send()
                .await
                .expect("Error while deleting contact list name\n");
            println!(
                "The specified contact list name '{}' has been deleted successfully",
                contact_list_name.green().bold()
            );
        } else {
            println!(
                "The contact list named '{}' doesn't exist\n",
                contact_list_name.red().bold()
            );
            println!(
                "{}\n",
                "The following lists are in your credentials and region"
                    .yellow()
                    .bold()
            );
            let contact_lists = self.list_contact_lists().await;
            for contact_list in contact_lists {
                println!("    {}", contact_list);
            }
            println!("");
        }
    }

    pub async fn create_email_identity(&self, email: &str) {
        let config = self.get_config();
        let client = SesClient::new(config);
        let available_email_identities = self.retrieve_emails_from_list_email_identities().await;
        if !available_email_identities.contains(email) {
            client
                .create_email_identity()
                .email_identity(email)
                .send()
                .await
                .expect("Error while creating Email Identity\n");
            let colored_email = email.green().bold();
            println!(
                "The email verfication send to: {} if exist\n",
                colored_email
            );
        } else {
            println!("The email identity '{}' already exists, but an email verification has been sent to this email again\n",email.yellow().bold());
            client
                .delete_email_identity()
                .email_identity(email)
                .send()
                .await
                .expect("Error while deleting Email Identity\n");
            client
                .create_email_identity()
                .email_identity(email)
                .send()
                .await
                .expect("Error while creating Email Identity\n");
            let colored_email = email.green().bold();
            println!("The email verfication send to: {}\n", colored_email);
        }
    }
    pub async fn delete_email_identity(&self, identity: &str) {
        let config = self.get_config();
        let client = SesClient::new(config);
        let available_email_identities = self.retrieve_emails_from_list_email_identities().await;
        if available_email_identities.contains(identity) {
            client
                .delete_email_identity()
                .email_identity(identity)
                .send()
                .await
                .expect("Error while deleting Email Identity\n");
            println!(
                "The provided email identity '{}' has been deleted\n",
                identity.green().bold()
            );
        } else {
            println!(
                "The email idenitity '{}' does not exist",
                identity.red().bold()
            );
            println!(
                "{}\n",
                "We will create an email identity for this address by sending a verification email"
                    .yellow()
                    .bold()
            );
            self.create_email_identity(identity).await;
        }
    }
    pub async fn delete_contact(&self, email: &str, list_name: Option<&str>, write_info: bool) {
        let config = self.get_config();
        let client = SesClient::new(config);
        if self
            .is_contact_list_name_exist(list_name.unwrap_or(&self.get_list_name()))
            .await
        {
            let available_contacts = self.get_contacts_in_the_list(list_name).await;
            if !available_contacts.is_empty() {
                if available_contacts.contains(email) {
                    client
                        .delete_contact()
                        .contact_list_name(list_name.unwrap_or(&self.get_list_name()))
                        .email_address(email)
                        .send()
                        .await
                        .expect("Error while deleting Email Contact\n");
                    if write_info {
                        println!(
                            "The provided contact '{}' has been deleted successfully\n",
                            email.green().bold()
                        );
                    }
                } else {
                    println!(
                        "There is no contact named '{}' existing in the Contact List named '{}'",
                        email.red().bold(),
                        list_name.unwrap_or(&self.get_list_name()).yellow().bold()
                    );
                    println!("{}\n","Please execute the 'Retrieve emails from the provided list' option to identify the emails in your list".yellow().bold());
                }
            } else {
                println!(
                    "{}",
                    "There are no contact to delete because none are available"
                        .red()
                        .bold()
                );
                println!("{}\n","Please add emails using the 'Add an email to the list' option and then execute this task".yellow().bold());
            }
        } else {
            println!(
                "The provided Contact List Name '{}' doesn't exist",
                list_name.unwrap_or(&self.get_list_name()).red().bold()
            );
            println!(
                "{}\n",
                "The contact list name below is available in your credentials or region if any"
                    .yellow()
                    .bold()
            );
            let contact_lists = self.list_contact_lists().await;
            for contact_list in contact_lists {
                println!("    {}", contact_list.green().bold());
            }
            println!("");
        }
    }
    pub async fn delete_contacts(&self, list_name: Option<&str>) {
        if self
            .is_contact_list_name_exist(list_name.unwrap_or(&self.get_list_name()))
            .await
        {
            let avaialble_contacts = self.get_contacts_in_the_list(list_name).await;
            if !avaialble_contacts.is_empty() {
                for contact in avaialble_contacts.split(" ") {
                    if !contact.is_empty() {
                        let client = SesClient::new(self.get_config());
                        client
                            .delete_contact()
                            .contact_list_name(list_name.unwrap_or(&self.get_list_name()))
                            .email_address(contact)
                            .send()
                            .await
                            .expect("Error while deleting contacts\n");
                    }
                }
                println!(
                    "All the contacts have been deleted from the contact list named '{}'",
                    list_name.unwrap_or(&self.get_list_name()).green().bold()
                );
                println!("{}\n","You can verify this by executing 'Retrieve emails from the provided list' where you should receive an empty text or PDF file if you see this message".yellow().bold());
            } else {
                println!(
                    "{}",
                    "There are no contacts to delete because none are available"
                        .red()
                        .bold()
                );
                println!("{}\n","Please add emails using the 'Add an email to the list' option and then execute this task".yellow().bold());
            }
        } else {
            println!(
                "The provided Contact List Name '{}' doesn't exist",
                list_name.unwrap_or(&self.get_list_name()).red().bold()
            );
            println!(
                "{}\n",
                "The contact list name below is available in your credentials or region if any"
                    .yellow()
                    .bold()
            );
            let contact_lists = self.list_contact_lists().await;
            for contact_list in contact_lists {
                println!("    {}", contact_list.green().bold());
            }
            println!("");
        }
    }

    pub async fn get_emails_given_list_name(&self, list_name: Option<&str>) -> Option<String> {
        let config = self.get_config();
        let client = SesClient::new(config);

        if self
            .is_contact_list_name_exist(list_name.unwrap_or(&self.get_list_name().as_str()))
            .await
        {
            let mut emails = String::new();
            let list = client
                .list_contacts()
                .contact_list_name(list_name.unwrap_or(&self.get_list_name().as_str()))
                .send()
                .await
                .map(|contacts| {
                    let colored_list_name = list_name
                        .unwrap_or(&self.get_list_name().as_str())
                        .green()
                        .bold();
                    println!("List named {} is exist\n", colored_list_name);
                    contacts
                })
                .expect("Error from retrieve_emails_from_provided_list\n");
            let contacts = list.contacts().unwrap();
            contacts.into_iter().for_each(|contact| {
                let email = contact.email_address().unwrap_or_default();
                emails.push_str(email);
            });
            Some(emails)
        } else {
            println!(
                "The provided Contact List Name '{}' doesn't exist",
                list_name
                    .unwrap_or(self.get_list_name().as_str())
                    .red()
                    .bold()
            );
            None
        }
    }
    pub async fn get_contacts_in_the_list(&self, list_name: Option<&str>) -> String {
        let config = self.get_config();
        let client = SesClient::new(config);
        let output = client
            .list_contacts()
            .contact_list_name(list_name.unwrap_or(&self.get_list_name()))
            .send()
            .await
            .expect("Error while getting contact lists\n");
        let mut contacts = String::new();
        if let Some(contacts_) = output.contacts {
            contacts_.into_iter().for_each(|contact| {
                contacts.push_str(contact.email_address.unwrap_or_default().as_str());
                contacts.push(' ');
            });
        }
        contacts
    }
    /// This function utilizes a default list name if 'None' is passed as a parameter.
    /// It incorporates 'create_identity' internally to send a verification email.
    /// Due to my use of a trial version, I am unable to employ a custom verification
    ///  template; therefore, this feature has not been tested yet
    pub async fn create_email_contact_with_verification(
        &self,
        email: &str,
        list_name: Option<&str>,
    ) {
        let config = self.get_config();
        let client = SesClient::new(config);
        let default_list_name = match list_name {
            Some(list_name) => list_name.to_string(),
            None => self.get_list_name(),
        };
        let available_contacts = self.get_contacts_in_the_list(list_name).await;
        if !available_contacts.contains(email) {
            let client = client
                .create_contact()
                .contact_list_name(&default_list_name)
                .email_address(email);
            let colored_error_outside = "Error from create_email_contact_with_verification"
                .red()
                .bold();

            client
                .send()
                .await
                .map(|_| async {
                    let colored_email = email.green().bold();
                    let colored_list_name = default_list_name.green().bold();
                    println!(
                        "The email address {colored_email} has been added to the contact list named: {}\n",
                        colored_list_name
                    );
                    let email_identies = self.retrieve_emails_from_list_email_identities().await;
                    if !email_identies.contains(email){
                        self.create_email_identity(email).await;
                    }else {
                        println!("The email '{}' already has an email identity",email.yellow().bold());
                        println!("But we are sending a verification email again for this email\n");
                        let client = SesClient::new(self.get_config());
                        client.delete_email_identity().email_identity(email).send().await.expect("Error while deleting email identity\n");
                        client.create_email_identity().email_identity(email).send().await.expect("Error while creating email identity\n");
                        println!("The verification email has been sent to: {}",email.green().bold());
                    }
                })
                .expect(&colored_error_outside)
                .await;
        } else {
            println!(
                "The email contact '{}' already exists in the given list '{}'",
                email.yellow().bold(),
                default_list_name.yellow().bold()
            );
            println!("{}\n","Use the 'Create Email Identity' option to send a verification email to this address if that's what you want".yellow().bold());
        }
    }

    /// Sometimes, we may not want to verify it immediately; instead, we simply want
    /// to add it to the list. Later on, we can initiate the verification process
    /// using the is_email_verified method.
    pub async fn create_email_contact_without_verification(
        &self,
        email: &str,
        list_name: Option<&str>,
    ) {
        let config = self.get_config();
        let client = SesClient::new(config);

        let default_list_name = match list_name {
            Some(list_name) => list_name.to_string(),
            None => self.get_list_name(),
        };
        let contacts = self.get_contacts_in_the_list(list_name).await;
        if !contacts.contains(email) {
            let client = client
                .create_contact()
                .contact_list_name(&default_list_name)
                .email_address(email);

            let colored_error = "Error from create_email_contact_without_verification\n"
                .red()
                .bold();

            client.send().await.expect(&colored_error);

            let colored_email = email.green().bold();
            let colored_list_name = default_list_name.green().bold();
            println!("The email address {colored_email} has been added to the contact list named: {colored_list_name}\n");
            println!("You must pass the email '{}' to the 'Create Email Identity' option before sending an email to this address\n",email.yellow().bold());
        } else {
            println!(
                "The email contact '{}' already exists in the given list '{}'",
                email.yellow().bold(),
                default_list_name.yellow().bold()
            );
            println!("{}\n","Use the 'Create Email Identity' option to send a verification email to this address if that's what you want".yellow().bold());
        }
    }
    /// Returns Some of true or false if the identity is exist otherwise returns None.
    pub async fn is_email_verfied(&self, email: &str) -> Option<bool> {
        let config = self.get_config();
        let client = SesClient::new(config);
        let email_identies = self.retrieve_emails_from_list_email_identities().await;
        if email_identies.contains(email) {
            let client = client
                .get_email_identity()
                .email_identity(email)
                .send()
                .await
                .expect("Error while Creating Email Identity\n");
            if client.verified_for_sending_status() {
                Some(true)
            } else {
                Some(false)
            }
        } else {
            None
        }
    }
    pub async fn is_contact_list_name_exist(&self, contact_list_name: &str) -> bool {
        let available_list_names = self
            .list_contact_lists()
            .await
            .into_iter()
            .map(|to_string| {
                let mut add_space = to_string;
                add_space.push(' ');
                add_space
            })
            .collect::<String>();
        if available_list_names.contains(contact_list_name) {
            true
        } else {
            false
        }
    }
    /// This helper function retrieves the emails from the provided contact list name,
    /// stores them in a vector of strings, and then returns them to the caller.
    pub async fn retrieve_emails_from_provided_list(
        &self,
        list_name: Option<&str>,
    ) -> Option<Vec<String>> {
        let config = self.get_config();
        let client = SesClient::new(config);

        let default_list_name = match list_name {
            Some(list_name) => list_name.to_string(),
            None => self.get_list_name(),
        };
        if self.is_contact_list_name_exist(&default_list_name).await {
            let mut emails = Vec::new();
            let list = client
                .list_contacts()
                .contact_list_name(&default_list_name)
                .send()
                .await
                .map(|contacts| {
                    let colored_list_name = default_list_name.green().bold();
                    println!("List named {} is exist\n", colored_list_name);
                    contacts
                })
                .expect("Error from retrieve_emails_from_provided_list\n");
            let contacts = list.contacts().unwrap();
            contacts
                .into_iter()
                .map(|contact| contact.email_address().unwrap_or_default().into())
                .for_each(|email| {
                    emails.push(email);
                });
            Some(emails)
        } else {
            println!(
                "The provided list name '{}' doesn't exist",
                default_list_name.red().bold()
            );
            let available_list_names = self.list_contact_lists().await;
            println!(
                "{}\n",
                "Only the contact list names below are available on your credentials or region"
                    .yellow()
                    .bold()
            );
            for contact_list_name in available_list_names {
                println!("    {}", contact_list_name.bright_green().bold());
            }
            println!("");
            None
        }
    }
    pub async fn retrieve_emails_from_list_email_identities(&self) -> String {
        let config = self.get_config();
        let client = SesClient::new(config);
        let outputs = client
            .list_email_identities()
            .send()
            .await
            .expect("Error while getting emails from list email identities api\n");
        let mut string_of_email_identies = String::new();
        if let Some(identityinfo) = outputs.email_identities {
            identityinfo.into_iter().for_each(|details| {
                if let Some(identity_name) = details.identity_name {
                    string_of_email_identies.push_str(&identity_name);
                    string_of_email_identies.push_str(" ");
                }
            });
        }
        string_of_email_identies
    }
    /// Retrieve the emails from the provided contact list name and save them to the
    /// current directory for future use.
    pub async fn writing_email_addresses_from_provided_list_as_text_pdf(
        &self,
        list_name: Option<&str>,
    ) {
        let emails = self.retrieve_emails_from_provided_list(list_name).await;
        match emails {
            Some(emails) => {
                let email_identities = self.retrieve_emails_from_list_email_identities().await;
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .read(true)
                    .open("./emails.txt")
                    .unwrap();
                let headers = vec![
                    "Identity Type",
                    "Identity Name",
                    "Is Sending Enabled",
                    "Verification Status",
                ];
                let mut vector_of_email_with_status = Vec::new();
                writeln!(file, "Emails\n").unwrap();
                for email in emails {
                    writeln!(file, "{email}\n").unwrap();
                    if email_identities.contains(&email) {
                        let client = SesClient::new(self.get_config());
                        let info = client
                            .get_email_identity()
                            .email_identity(&email)
                            .send()
                            .await
                            .expect("Error while getting email identity\n");

                        if let Some(identity_type) = info.identity_type {
                            let type_ = identity_type.as_str().to_string();
                            let buf = format!("Identity Type: {type_}");
                            file.write_all(buf.as_bytes()).unwrap();
                            vector_of_email_with_status.push(type_);
                        }
                        vector_of_email_with_status.push(email);
                        let sending_enabled = format!("{}", info.verified_for_sending_status);
                        let buf = format!("Is Sending Enabled: {sending_enabled}");
                        file.write_all(buf.as_bytes()).unwrap();
                        vector_of_email_with_status.push(sending_enabled);

                        if let Some(status) = info.verification_status {
                            let status = status.as_str().to_string();
                            let buf = format!("Verification Status: {status}\n\n");
                            file.write_all(buf.as_bytes()).unwrap();
                            vector_of_email_with_status.push(status);
                        }
                    } else {
                        vector_of_email_with_status.push("No Identity Exists".into());
                        vector_of_email_with_status.push(email);
                        vector_of_email_with_status.push("No Identity Exists".into());
                        vector_of_email_with_status.push("No Identity Exists".into());
                    }
                }
                match File::open("emails.txt") {
                    Ok(_) => println!(
                        "{}\n",
                        "Emails are written to a file called 'emails.txt'\n To view the emails, please check the current directory".green().bold()
                    ),
                    Err(_) => println!("{}\n", "Error while writing file".red().bold()),
                }

                let get_list_name =
                    var("LIST_NAME").unwrap_or("No 'LIST_NAME' environment variable found".into());
                let list_name = list_name.unwrap_or(&get_list_name);
                let region_name = self
                    .config
                    .region()
                    .map(|region| region.as_ref())
                    .unwrap_or("No region is found in the Credential");
                create_email_pdf(
                    &headers,
                    vector_of_email_with_status,
                    list_name,
                    region_name,
                );
            }
            None => {}
        }
    }
    pub async fn writing_email_identies_details_as_text_pdf(&self) {
        let headers = vec![
            "Identity Type",
            "Identity Name",
            "Is Sending Enabled",
            "Verification Status",
        ];
        let identities = self.list_email_identity().await;
        let region_name = self
            .get_config()
            .region()
            .map(|region| region.as_ref())
            .unwrap_or("No Region is found");
        create_email_identities_pdf(&headers, identities, region_name);
    }

    /// This only works when production access is enabled, i.e., in a paid AWS service, instead of a trial version that has not been tested.
    pub async fn send_custom_verification(&self, email: &str, template_name: &str) {
        let config = self.get_config();
        let client = SesClient::new(config);

        let send = client
            .send_custom_verification_email()
            .email_address(email)
            .template_name(template_name);
        match send.send().await {
            Ok(_) => {
                let colored_email = email.green().bold();
                println!("Mail verfication is send to : {colored_email}\n")
            }
            Err(_) => {
                let colored_email = email.red().bold();
                println!("Error while sending Verfication to : {colored_email}\n")
            }
        }
    }
    pub async fn create_email_template(
        &self,
        template_name: &str,
        subject: &str,
        template: &str,
        text: Option<String>,
    ) {
        let config = self.get_config();
        let client = SesClient::new(config);
        if !self.is_email_template_exist(template_name).await {
            let email_template_builder = EmailTemplateContent::builder()
                .subject(subject)
                .html(template)
                .set_text(text)
                .build();
            let build = client
                .create_email_template()
                .template_content(email_template_builder)
                .template_name(template_name);

            let colored_error = "Error from create_email_template\n".red().bold();
            build
                .send()
                .await
                .map(|_| {
                    let colored_tempname = template_name.green().bold();
                    println!(
                        "The email template named '{}' has been created\n",
                        colored_tempname,
                    )
                })
                .expect(&colored_error);
        } else {
            println!("Template '{}' already exists", template_name.red().bold());
            println!("{}", "Try using different template name".yellow().bold());
            println!(
                "{}\n",
                "Below are the available template names in your credentials or region"
                    .yellow()
                    .bold()
            );
            let templates = self.list_email_templates().await;
            for template_name in templates {
                println!("    {}", template_name.green().bold());
            }
            println!("");
        }
    }
    pub async fn list_email_templates(&self) -> Vec<String> {
        let config = self.get_config();
        let client = SesClient::new(config);
        let outputs = client
            .list_email_templates()
            .send()
            .await
            .expect("Error while getting Email Templates\n");
        let mut templates_names = Vec::new();
        if let Some(template_meta_data) = outputs.templates_metadata {
            template_meta_data.into_iter().for_each(|template_detail| {
                if let Some(temp_name) = template_detail.template_name {
                    templates_names.push(temp_name);
                }
            });
        }
        templates_names
    }
    pub async fn is_email_template_exist(&self, template_name: &str) -> bool {
        let email_templates = self
            .list_email_templates()
            .await
            .into_iter()
            .map(|to_string| {
                let mut add_space = to_string;
                add_space.push(' ');
                add_space
            })
            .collect::<String>();
        if email_templates.contains(template_name) {
            true
        } else {
            false
        }
    }
    pub async fn delete_template(&self, template_name: &str) {
        let config = self.get_config();
        let client = SesClient::new(config);
        if self.is_email_template_exist(template_name).await {
            client
                .delete_email_template()
                .template_name(template_name)
                .send()
                .await
                .expect("Error While deleting Template\n");
            println!("The template associated with the specified template name '{}' has been deleted successfully",template_name.green().bold());
        } else {
            println!(
                "The template named '{}' doesn't exist",
                template_name.red().bold()
            );
            println!(
                "{}\n",
                "Here are the available template names in your credentials or region"
                    .yellow()
                    .bold()
            );
            let templates = self.list_email_templates().await;
            for template_name in templates {
                println!("    {}", template_name.green().bold());
            }
            println!("");
        }
    }
    pub async fn update_template(
        &self,
        template_name: &str,
        subject: &str,
        html: &str,
        text: Option<String>,
    ) {
        if self.is_email_template_exist(template_name).await {
            let config = self.get_config();
            let client = SesClient::new(config);
            let text_str = text
                .as_ref()
                .map(|to_string| to_string.to_string())
                .unwrap_or("".into());
            let template_builder = EmailTemplateContent::builder()
                .subject(subject)
                .set_text(text)
                .html(html)
                .build();
            client
                .update_email_template()
                .template_name(template_name)
                .template_content(template_builder)
                .send()
                .await
                .expect("Error while Updating Template\n");
            match (subject.is_empty(),text_str.is_empty()) {
            (false,false) => println!("{}\n","The template has been successfully updated with the provided subject, HTML body, and text body".green().bold()),
            (true,false) => println!("{}\n","The template has been successfully updated with the provided HTML body, and text body".green().bold()),
            (false,true) => println!("{}\n","The template has been successfully updated with the provided subject, HTML body".green().bold()),
            (true,true) => println!("{}\n","The template has been successfully updated with the provided HTML body".green().bold())
        }
        } else {
            println!(
                "The template named '{}' doesn't exist",
                template_name.red().bold()
            );
            println!(
                "{}\n",
                "Here are the available template names in your credentials or region"
                    .yellow()
                    .bold()
            );
            let templates = self.list_email_templates().await;
            for template_name in templates {
                println!("    {}", template_name.green().bold());
            }
            println!("");
        }
    }
    pub async fn get_template_subject_html_and_text(
        &self,
        template_name: &str,
        write_info: bool,
    ) -> Option<(String, String, String)> {
        if self.is_email_template_exist(template_name).await {
            let mut subject = String::new();
            let mut html = String::new();
            let mut text = String::new();
            let config = self.get_config();
            let client = SesClient::new(config);
            let outputs = client
                .get_email_template()
                .template_name(template_name)
                .send()
                .await
                .expect("Error While Getting Template\n");
            let file_name = format!("EmailTemplateOf{template_name}.html");
            let mut email_template = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(&file_name)
                .expect("Error while creating file for Email Template Content\n");
            if let Some(content) = outputs.template_content {
                if let Some(subject_) = content.subject {
                    subject.push_str(&subject_);
                    if write_info {
                        let buf = format!(
                            r#"<h1 style="text-align: center;">Subject Part</h1><br><br><p style="text-align: center;padding-left: 100px;padding-right: 100px;">{subject_}</p><br><br>"#
                        );
                        email_template.write_all(buf.as_bytes()).unwrap();
                    }
                }
                if let Some(html_) = content.html {
                    html.push_str(&html_);
                    if write_info {
                        let buf = format!(
                            r#"<h1 style="text-align: center;">Html Part</h1><br><br>{html_}<br><br>"#
                        );
                        email_template.write_all(buf.as_bytes()).unwrap();
                    }
                }
                if let Some(text_) = content.text {
                    text.push_str(&text_);
                    if write_info {
                        let buf = format!(
                            r#"<h1 style="text-align: center;">Text Part</h1><br><br><p style="padding-left: 100px;padding-right: 100px;">{text_}</p><br><br><br><br><br>"#
                        );
                        email_template.write_all(buf.as_bytes()).unwrap();
                    }
                }
            }
            if write_info {
                match File::open(&file_name) {
                Ok(_) => println!("The email template associated with the template name '{}' has been successfully created in the current directory with the file name '{}{}.{}'\n",template_name.green().bold(),"EmailTemplateOf".green().bold(),template_name.green().bold(),"html".green().bold()),
                Err(_) => println!("Error While writing Email Template\n")
            }
            } else {
                std::fs::remove_file(&file_name).expect("This won't fail unless in Rare cases\n");
            }
            Some((subject, html, text))
        } else {
            println!(
                "The template named '{}' doesn't exist",
                template_name.red().bold()
            );
            println!(
                "{}\n",
                "Here are the available template names in your credentials or region"
                    .yellow()
                    .bold()
            );
            let templates = self.list_email_templates().await;
            for template_name in templates {
                println!("    {}", template_name.green().bold());
            }
            println!("");
            None
        }
    }
    pub fn get_template_variables_of_subject_and_html_body(
        &self,
        subject: &str,
        template: &str,
    ) -> (Vec<String>, Vec<String>) {
        let pattern = r#"\{\{.*?\}\}"#;
        let html_variables = Regex::new(pattern).expect("Error while parsing Regex Syntax\n");
        let subject_variables = Regex::new(pattern).expect("Error while parsing Regex Syntax\n");
        (
            subject_variables
                .find_iter(subject)
                .map(|data| data.as_str().to_string())
                .collect(),
            html_variables
                .find_iter(template)
                .map(|data| data.as_str().to_string())
                .collect(),
        )
    }
    pub async fn match_template_data_with_template(
        &self,
        template_name: Option<&str>,
        template_data_path: &str,
    ) {
        let json_keys_pattern = regex::Regex::new(r#""(\w+)"\s*:"#);
        let template_josn_data = fs::read_to_string(template_data_path)
            .expect("Error while opening the template data path you specified\n");
        match json_keys_pattern {
            Ok(regex) => {
                let keys = regex
                    .find_iter(&template_josn_data)
                    .map(|to_str| {
                        let remove_quotes = to_str.as_str().to_string();
                        // println!("{}\n",remove_quotes);
                        let matches: &[_] = &['"', ':', ' '];
                        remove_quotes.trim_matches(matches).to_string()
                    })
                    .collect::<Vec<String>>();
                //println!("Template Data Json Keys:\n{}\n", keys.join("\n"));
                let get_template = self
                    .get_template_subject_html_and_text(
                        template_name.unwrap_or(&self.get_template_name()),
                        false,
                    )
                    .await;
                match get_template {
                    Some((subject, html, _)) => {
                        let (subject_variables, html_variables) =
                            self.get_template_variables_of_subject_and_html_body(&subject, &html);
                        let subject_variables = subject_variables
                            .into_iter()
                            .map(|to_string| {
                                let mut add_space = to_string;
                                add_space.push(' ');
                                add_space
                            })
                            .collect::<String>();
                        let html_variables = html_variables
                            .into_iter()
                            .map(|to_string| {
                                let mut add_space = to_string;
                                add_space.push(' ');
                                add_space
                            })
                            .collect::<String>();
                        let template_variables = subject_variables + &html_variables;
                        for variable in keys {
                            if template_variables.contains(&variable) {
                                println!("The template data variable {} in the specified JSON document matches the template variable of template {}", variable.green().bold(),template_name.unwrap_or(&self.get_template_name()).yellow().bold());
                            } else {
                                println!("The variable in the template {} in the given JSON document does not match the variable in the template named {}\n",variable.red().bold(),template_name.unwrap_or(&self.get_template_name()).yellow().bold());
                            }
                        }
                        println!("{}\n","This option will not match the spaces around the keys. Execute the 'Get Email Template Variables' option to see the template variables".yellow().bold());
                    }
                    None => {}
                }
            }
            Err(_) => println!(
                "{}\n",
                "Error while parsing the template json you specified"
                    .red()
                    .bold()
            ),
        }
    }
    /// Create a helper function for sending single emails, allowing other parts of the code or users to customize it for sending bulk emails
    pub async fn send_mono_email(
        &self,
        email: &str,
        simple_or_template: SimpleOrTemplate,
        from_address: Option<&str>,
    ) -> Result<SendEmailFluentBuilder, String> {
        let client = SesClient::new(self.get_config());
        let email_identies = self.retrieve_emails_from_list_email_identities().await;
        if email_identies.contains(email) {
            let is_email_verified = self.is_email_verfied(&email).await;
            match is_email_verified {
                Some(status) => {
                    if status {
                        let destination = Destination::builder().to_addresses(email).build();
                        let default_from_address = self.get_from_address();
                        let from_address = from_address.unwrap_or(&default_from_address);
                        match simple_or_template {
                            Simple_(simple) => Ok(client
                                .send_email()
                                .content(simple)
                                .from_email_address(from_address)
                                .destination(destination)),
                            Template_(template) => Ok(client
                                .send_email()
                                .content(template)
                                .from_email_address(from_address)
                                .destination(destination)),
                        }
                    } else {
                        let why_failed = format!("The email ---{}--- has not been verified; we have re-sent the verification email",email);
                        client
                            .delete_email_identity()
                            .email_identity(email)
                            .send()
                            .await
                            .expect("Error while deleting email identity\n");
                        client
                            .create_email_identity()
                            .email_identity(email)
                            .send()
                            .await
                            .expect("Error while creating email identity\n");

                        Err(why_failed)
                    }
                }
                None => {
                    let why_failed = format!("The email identity '{}' doesn't exist.\nThis email should be verified through 'create email identity' option before sending an email",email);
                    Err(why_failed)
                }
            }
        } else {
            let why_failed = format!("The email identity '{}' doesn't exist.\nThis email should be verified through 'create email identity' option before sending an email",email);
            Err(why_failed)
        }
    }

    /// A helpful utility function I've created for myself is designed to send templated
    /// emails to the addresses in a list, all without introducing any code smells on
    /// the caller's side and doesn't take any parameters. This is inlcuded for your reference
    /// Here is the [`template`](https://tinyurl.com/4ssuz7fy) I've used for this operation.
    pub async fn send_bulk_templated_emails(&self) {
        let emails = self
            .retrieve_emails_from_provided_list(Some(&self.get_list_name()))
            .await;
        match emails {
            Some(emails) => {
                let email_identies = self.retrieve_emails_from_list_email_identities().await;
                let load_json = include_str!("./assets/template_data.json").to_string();
                'go: for email in emails.iter() {
                    if email_identies.contains(email) {
                        let is_email_verified = self.is_email_verfied(&email).await;
                        match is_email_verified {
                            Some(status) => {
                                if status {
                                    let name = email.chars().take(9).collect::<String>();
                                    let data = load_json.replace("email", email);
                                    let data = data.replace("name", &name);
                                    let template = TemplateMail::builder(
                                        self.get_template_name().as_str(),
                                        &data,
                                    )
                                    .build();
                                    match self
                                        .send_mono_email(
                                            email,
                                            Template_(template),
                                            Some(&self.get_from_address()),
                                        )
                                        .await
                                    {
                                        Ok(email_builder) => {
                                            email_builder
                                    .send()
                                    .await
                                    .expect("Error while executing Send_bulk_templated_emails\n");
                                            let colored_email = email.green().bold();
                                            let colored_template_data = data.green().bold();
                                            println!("The template mail is send to: {colored_email} \nand the template data is: {colored_template_data}\n");
                                        }
                                        Err(msg) => println!("{}", msg),
                                    }
                                } else {
                                    println!("The email address '{}' in the list hasn't been verified, yet it continues to send templated emails to other verified email addresses in the list\n",email.bright_red().bold());
                                    continue 'go;
                                }
                            }
                            None => {
                                println!(
                                    "The email identity for '{}' does not exist",
                                    email.red().bold()
                                );
                                println!("{}\n","Please use the 'Create Email Identity' option or function to establish an identity for this email".yellow().bold());
                            }
                        }
                    } else {
                        println!("The email address '{}' in this list doesn't have an identity; therefore, it creates an identity by executing the 'Create Email Identity' option on your behalf\n",email.yellow().bold());
                        self.create_email_identity(email).await;
                    }
                }
                println!("{}","If you have any red-colored emails above, the operation won't be executed for those emails, but it will be executed for the other emails, where the templated mail has already been sent".yellow().bold());
            }
            None => {
                println!(
                    "The provided list name '{}' doesn't exist",
                    self.get_list_name().red().bold()
                );
            }
        }
    }

    /// This method accept type of `SimpleMail` with content of [`EmailContent`](https://docs.rs/aws-sdk-sesv2/latest/aws_sdk_sesv2/types/struct.EmailContent.html)
    pub async fn send_multi_email_with_simple(
        &self,
        data: SimpleMail,
        from_address: Option<&str>,
        list_name: Option<&str>,
    ) {
        let colored_error = "Error from send_multi_email_with_simple function"
            .red()
            .bold();

        let emails = self.retrieve_emails_from_provided_list(list_name).await;
        match emails {
            Some(emails) => {
                let email_content = data.build();
                let email_identies = self.retrieve_emails_from_list_email_identities().await;
                'go: for email in emails.into_iter() {
                    if email_identies.contains(&email) {
                        let is_email_verified = self.is_email_verfied(&email).await;
                        match is_email_verified {
                            Some(status) => {
                                if status {
                                    let email_content_ = email_content.clone();
                                    match self
                                        .send_mono_email(
                                            &email,
                                            Simple_(email_content_),
                                            from_address,
                                        )
                                        .await
                                    {
                                        Ok(email_builder) => {
                                            email_builder
                                        .send()
                                        .await
                                        .map(|_| {
                                            let colored_email = email.green().bold();
                                            println!(
                                                "Simple Email Content is send to {colored_email} successfully\n"
                                            )
                                        })
                                        .expect(&colored_error);
                                        }
                                        Err(msg) => println!("{}", msg),
                                    }
                                } else {
                                    println!("The email address '{}' in the list hasn't been verified, yet it continues to send Simple Emails to other verified email addresses in the list\n",email.bright_red().bold());
                                    continue 'go;
                                }
                            }
                            None => {
                                println!(
                                    "The email identity for '{}' does not exist",
                                    email.red().bold()
                                );
                                println!("{}\n","Please use the 'Create Email Identity' option or function to establish an identity for this email".yellow().bold());
                            }
                        }
                    } else {
                        println!("The email address '{}' in this list doesn't have an identity; therefore, it creates an identity by executing the 'Create Email Identity' option on your behalf\n",email.yellow().bold());
                        self.create_email_identity(&email).await;
                    }
                }
                println!("{}\n","If you have any red-colored emails above, the operation won't be executed for those emails, but it will be executed for the other emails, where the Simple Mail has already been sent".yellow().bold());
            }
            None => {
                println!(
                    "The provided list name '{}' doesn't exist",
                    self.get_list_name().red().bold()
                );
            }
        }
    }
}

/// Types and methods for creating a straightforward email template with essential user
/// information, requiring minimal code to generate the content. This aligns with
/// the expectations of a template or simple email operation. After configuring
/// the builder, you can call the 'build' method to obtain the appropriate type.
/// This approach is reminiscent of how AWS APIs are designed.
/// This wrapped enum type is used to reduce boilerplate code
#[derive(Clone)]
pub enum SimpleOrTemplate {
    Simple_(EmailContent),
    Template_(EmailContent),
}
pub struct SimpleMail {
    body: String,
    subject: String,
}

impl SimpleMail {
    pub fn builder(body: &str, subject: &str) -> Self {
        Self {
            body: body.into(),
            subject: subject.into(),
        }
    }
    pub fn build(self) -> EmailContent {
        let subject_content = Content::builder()
            .charset("UTF-8")
            .data(self.subject)
            .build();

        let body_content = Content::builder().charset("UTF-8").data(self.body).build();

        let body = Body::builder().html(body_content).build();
        let message = Message::builder()
            .body(body)
            .subject(subject_content)
            .build();

        EmailContent::builder().simple(message).build()
    }
}

pub struct TemplateMail<'a> {
    template_name: &'a str,
    template_data: &'a str,
}

impl<'a> TemplateMail<'a> {
    pub fn builder(template_name: &'a str, template_data: &'a str) -> Self {
        Self {
            template_name: template_name,
            template_data: template_data,
        }
    }
    pub fn build(self) -> EmailContent {
        let template = Template::builder()
            .template_name(self.template_name)
            .template_data(self.template_data)
            //.template_arn("your_template_arn")
            .build();

        EmailContent::builder().template(template).build()
    }
}
