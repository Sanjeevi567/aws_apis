use self::SimpleOrTemplate::{Simple_, Template_};
use aws_config::SdkConfig;
use aws_sdk_sesv2 as sesv2;
use colored::Colorize;
use sesv2::{
    operation::{
        create_email_identity::builders::CreateEmailIdentityFluentBuilder,
        send_email::builders::SendEmailFluentBuilder,
    },
    types::{Body, Content, Destination, EmailContent, EmailTemplateContent, Message, Template},
    Client as SesClient,
};
use std::{fs::OpenOptions, io::Write, thread::sleep, time::Duration};

/// The core structure for performing operations on [`SESv2`](https://docs.rs/aws-sdk-sesv2/latest/aws_sdk_sesv2/struct.Client.html) (Simple Email Service Version 2)
/// clients eliminates the need for users of the API to provide credentials each
/// time they use the service. Instead, these credentials are abstracted by this
/// structure along with its inherent functions and methods.
#[derive(Debug)]
pub struct SesOps {
    config: SdkConfig,
    list_name: Option<String>,
    template_name: Option<String>,
    from_address: Option<String>,
}

impl SesOps {
    ///When calling this function, it builds the credentials and includes default values such as
    /// 'from_address' and 'list_name' for use in other services. This means you don't have to pass
    /// them explicitly every time, but you can modify them through methods like 'set_
    pub fn build(config: SdkConfig) -> Self {
        Self {
            config: config,
            //list name must be exist
            list_name: Some("".into()),
            //Template name must exist
            template_name: Some("".into()),
            from_address: Some("".into()),
        }
    }
    ///Getting default values directly from the struct itself, not as actual data
    /// retrieved from an AWS service. In other words, these values act as proxies for the actual data if you're familiar with these details
    pub fn get_from_address(&self) -> String {
        self.from_address.as_deref().unwrap_or("You can set the default from_address by selecting the 'configure' option from the menu").into()
    }
    pub fn get_template_name(&self) -> &str {
        self.template_name.as_deref().unwrap_or("You can set the default template name by selecting the 'configure' option from the menu")
    }
    pub fn get_list_name(&self) -> &str {
        self.list_name.as_deref().unwrap_or("You can set the default list name by selecting the 'configure' option from the menu")
    }

    /// If the list name does not exist, i.e., if it has not been set using the
    /// appropriate methods, an error will occur when attempting to use it.
    pub fn set_list_name(&mut self, list_name: &str) {
        self.list_name = Some(list_name.into());
    }
    /// The template name must correspond to the credentials you used, and the
    /// template data must accurately match the template employed by those services
    pub fn set_template_name(&mut self, template_name: &str) {
        self.template_name = Some(template_name.into());
    }
    /// The 'from' address has to be verified since this is the base email used to send mail to others
    pub fn set_from_address(&mut self, from_address: &str) {
        self.from_address = Some(from_address.into());
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
    }

    /// The 'create email identity' helper function is isolated, so we don't have to use it unless necessary.
    async fn create_email_identity(&self, email: &str) -> CreateEmailIdentityFluentBuilder {
        let config = self.get_config();
        let client = SesClient::new(config);

        client.create_email_identity().email_identity(email)
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

        let default_list_name = list_name.unwrap_or(self.get_list_name());
        let client = client
            .create_contact()
            .contact_list_name(default_list_name)
            .email_address(email);

        let colored_error_inside = "Error from create_email_identity inside of create_email_contact_with_verification".red().bold();
        let colored_error_outside = "Error from create_email_contact_with_verification".red().bold();
        

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
                self.create_email_identity(email)
                    .await
                    .send()
                    .await
                    .map(|_| {
                    let colored_email = email.green().bold();
                    println!("The email verfication send to: {} if exist\n", colored_email)
                     })
                    .expect(&colored_error_inside);
            })
            .expect(&colored_error_outside)
            .await;
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

        let default_list_name = list_name.unwrap_or(self.get_list_name());
        let client = client
            .create_contact()
            .contact_list_name(default_list_name)
            .email_address(email);

        let colored_error = "Error from create_email_contact_without_verification".red().bold();

        client
    .send()
    .await
    .map(|_|{
        let colored_email = email.green().bold();
        let colored_list_name = default_list_name.green().bold();
        println!("The email address {colored_email} has been added to the contact list named: {colored_list_name}\n")
    })
    .expect(&colored_error);
    }
    /// Returns true if the email is verified; otherwise, returns false.
    pub async fn is_email_verfied(&self, email: &str) -> bool {
        let config = self.get_config();
        let client = SesClient::new(config);

        let client = client
            .get_email_identity()
            .email_identity(email)
            .send()
            .await
            .unwrap();

        if client.verified_for_sending_status() {
            true
        } else {
            false
        }
    }
    /// This helper function retrieves the emails from the provided contact list name,
    /// stores them in a vector of strings, and then returns them to the caller.
    async fn retrieve_emails_from_provided_list(&self, list_name: Option<&str>) -> Vec<String> {
        let config = self.get_config();
        let client = SesClient::new(config);

        let colored_error = "Error from retrieve_emails_from_provided_list".red().bold();

        let default_list_name = list_name.unwrap_or(self.get_list_name());

        
        let list = client
            .list_contacts()
            .contact_list_name(default_list_name)
            .send()
            .await
            .map(|contacts| {
                let colored_list_name = default_list_name.green().bold();
                println!("List named {} is exist\n", colored_list_name);
                println!("{}\n","Data is retrieved from the internet, a process that takes seconds.".blue().bold());
                contacts
            })
            .expect(&colored_error);
        let contacts = list.contacts().unwrap();
        contacts
            .into_iter()
            .map(|contact| contact.email_address().unwrap_or_default().into())
            .collect()
    }

    /// Retrieve the emails from the provided contact list name and save them to the
    /// current directory for future use.
    pub async fn getting_email_addresses_from_provided_list(
        &self,
        list_name: Option<&str>,
        print_emails: bool,
        upto: Option<usize>,
    ) {
        let emails = self.retrieve_emails_from_provided_list(list_name).await;

        if print_emails {
            let mut count = 0;
            for email in &emails {
                if count != upto.unwrap_or(0) {
                    let email = email.green().bold();
                    println!("    {}\n", email);
                    count += 1;
                    sleep(Duration::from_millis(1000));
                }
            }
        }
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open("./emails.txt")
            .unwrap();
        emails
            .iter()
            .for_each(|email| writeln!(file, "{}\n", email).unwrap());
        println!(
            "{}\n",
            "Emails are written to a file called 'emails.txt.'\n To view the emails, please check the current directory".green().bold()
        );
    }

    /// This only works when production access is enabled, i.e., in a paid AWS service, instead of a trial version that has not been tested.

    pub async fn send_custom_verification(&self, email: &str) {
        let config = self.get_config();
        let client = SesClient::new(config);

        let send = client
            .send_custom_verification_email()
            .email_address(email)
            .template_name("Verification");
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
    /// Store the template in a separate HTML file with template variables formatted as {{variable_name}}, and retrieve them using the include_str! macro.
    /// The number of template variables used is also passed when sending templated emails unless all the variables have default values.

    pub async fn create_email_template(&self, template_name: &str, template: &str, subject: &str) {
        let config = self.get_config();
        let client = SesClient::new(config);

        let email_template_builder = EmailTemplateContent::builder()
            .subject(subject)
            .set_text(None)
            .html(template)
            .build();
        let build = client
            .create_email_template()
            .template_content(email_template_builder)
            .template_name(template_name);

     let colored_error = "Error from create_email_template".red().bold();
        build
            .send()
            .await
            .map(|_| {
                let colored_tempname = template_name.green().bold();
                let colored_template = template.green().bold();
                println!(
                    "Email Template named {} and the template data {} is created\n",
                    colored_tempname, colored_template
                )
            })
            .expect(&colored_error);
    }

    /// Create a helper function for sending single emails, allowing other parts of the code or users to customize it for sending bulk emails
    pub async fn send_mono_email(
        &self,
        email: &str,
        simple_or_template: SimpleOrTemplate,
        from_address: Option<&str>,
    ) -> SendEmailFluentBuilder {
        let client = SesClient::new(self.get_config());

        let email_address = vec![String::from(email)];

        let destination = Destination::builder()
            .set_to_addresses(Some(email_address))
            .build();
        let default_from_address = self.get_from_address();
        let from_address = from_address.unwrap_or(&default_from_address);
        match simple_or_template {
            Simple_(simple) => client
                .send_email()
                .content(simple)
                .from_email_address(from_address)
                .destination(destination),
            Template_(template) => client
                .send_email()
                .content(template)
                .from_email_address(from_address)
                .destination(destination),
        }
    }

    /// A helpful utility function I've created for myself is designed to send templated
    /// emails to the addresses in a list, all without introducing any code smells on
    /// the caller's side and doesn't take any parameters. This is inlcuded for your reference
    pub async fn send_emails(&self) {

        let colored_error = "Error from send_emails function".red().bold();
        let emails = self
            .retrieve_emails_from_provided_list(Some(self.get_list_name()))
            .await;

        for email in emails.iter() {
            let name = email.chars().take(9).collect::<String>();
            //Author's template variables that lack default values
            let template_data = format!(
                "
             {{
              \"Name\": \"{}\",
              \"Email\" : \"{}\"
             }}
            ",
                name, email
            );
            
            let template = TemplateMail::builder(self.get_template_name(), &template_data).build();
            self.send_mono_email(email, Template_(template), Some(&self.get_from_address()))
                .await
                .send()
                .await
                .map(|_|{
                    let colored_email = email.green().bold();
                    let colored_template_data = template_data.green().bold();
     println!("The template mail is send to: {colored_email} \nand the template data is: {colored_template_data}\n")
        })
                .expect(&colored_error);
        }
    }
    /// Here is the [`template`]() I've used for this operation.

    /// This method accept type of `SimpleMail` with content of [`EmailContent`](https://docs.rs/aws-sdk-sesv2/latest/aws_sdk_sesv2/types/struct.EmailContent.html)
    pub async fn send_multi_email_with_simple(
        &self,
        data: SimpleMail,
        from_address: Option<&str>,
        list_name: Option<&str>,
    ) {

        let colored_error = "Error from send_multi_email_with_simple function".red().bold();

        let emails = self.retrieve_emails_from_provided_list(list_name).await;
  
        let email_content = data.build();

        for email in emails.into_iter() {
            let email_content_ = email_content.clone();
            //println!("{email}\n");
            self.send_mono_email(&email, Simple_(email_content_), from_address)
                .await
                .send()
                .await
                .map(|_| {
                    let colored_email = email.green().bold();
                    println!("Simple Mail is send to {colored_email} successfully...\n")
                })
                .expect(&colored_error);
        }
    }

    /// This utility function is designed for sending mail to a single address.
    /// It becomes particularly useful when you have multiple clients and need to send distinct data
    /// using the same template, possibly with the assistance of machine learning algorithms for suggestions.

    pub async fn send_multi_email_with_template(
        &self,
        data: TemplateMail<'static>,
        from_address: Option<&str>,
        list_name: Option<&str>,
    ) {
        let emails = self.retrieve_emails_from_provided_list(list_name).await;

        let email_content = data.build();

        let colored_error = "Error from send_multi_email_with_template".red().bold();
        for email in emails.into_iter() {
            let email_content_ = email_content.clone();
            self.send_mono_email(&email, Simple_(email_content_), from_address)
                .await
                .send()
                .await
                .map(|_| {
                    let colored_email = email.green().bold();
                    println!("Template Mail is send to {colored_email} successfully...\n")
                })
                .expect(&colored_error);
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

        let body = Body::builder().text(body_content).build();
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