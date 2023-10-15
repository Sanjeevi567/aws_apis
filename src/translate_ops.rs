use aws_config::SdkConfig;
use aws_credential_types::provider::ProvideCredentials;
use aws_sdk_translate::{
    primitives::{Blob, DateTimeFormat},
    types::{Document, InputDataConfig, OutputDataConfig},
    Client as TranslateClient,
};
use colored::Colorize;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
};

use crate::{create_translated_text_pdf, create_translation_language_details_pdf};
pub struct TranslateOps<'a> {
    config: &'a SdkConfig,
}
impl<'a> TranslateOps<'a> {
    pub fn build(config: &'a SdkConfig) -> Self {
        Self { config }
    }
    pub async fn list_languages(&self, write_info: bool) -> (Vec<String>, Vec<String>) {
        let client = TranslateClient::new(self.config);
        let outputs = client
            .list_languages()
            .send()
            .await
            .expect("Error while Listing Languages\n");
        let mut lang_names = Vec::new();
        let mut lang_codes = Vec::new();
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("Supported_Languages_Details.txt")
            .expect("Error while creating file\n");
        if let Some(language) = outputs.languages {
            language.into_iter().for_each(|details| {
                if let Some(lang_name) = details.language_name {
                    let buf = format!("Language Name: {}\n", lang_name);
                    lang_names.push(lang_name);
                    file.write_all(buf.as_bytes()).unwrap();
                }
                if let Some(lang_code) = details.language_code {
                    let buf = format!("Language Code: {}\n\n", lang_code);
                    lang_codes.push(lang_code);
                    file.write_all(buf.as_bytes()).unwrap();
                }
            });
        }
        if write_info {
            match File::open("Supported_Languages_Details.txt") {
                Ok(_) => println!("The supported language details have been saved to the current directory with the filename '{}'\n","Supported_Languages_Details.txt".green().bold()),
                Err(_) => println!("{}\n","Error while writing Data".red().bold())
            }
            create_translation_language_details_pdf(lang_names.clone(), lang_codes.clone());
        }

        (lang_names, lang_codes)
    }
    pub async fn translate_text(&self, text_path: &str, target_lang_code: &str) {
        let client = TranslateClient::new(self.config);
        let text_data =
            fs::read_to_string(text_path).expect("Error opening the file you specified\n");
        let outputs = client
            .translate_text()
            .source_language_code("auto")
            .text(text_data)
            .target_language_code(target_lang_code)
            .send()
            .await
            .expect("Error while Translate Text\n");
        if let Some(translated_text) = outputs.translated_text {
            let mut file = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open("TranslatedText.txt")
                .expect("Error while creating file\n");
            let buf = format!(
                "The translation for the provided text:\n{}",
                translated_text
            );
            file.write_all(buf.as_bytes()).unwrap();
            match File::open("TranslatedText.txt") {
                Ok(_) => println!("The translated text for the given input is saved in the current directory under the filename '{}'\n","TranslatedText.txt".green().bold()),
                Err(_) => println!("{}\n","Error while opening the file".red().bold()),
            }
            create_translated_text_pdf(translated_text);
        }
    }
    pub async fn translate_document(
        &self,
        document_type: &str,
        document_path: &str,
        target_lang_code: &str,
    ) {
        let client = TranslateClient::new(self.config);
        let document_type_ = match document_type {
            "html" | "HTML" | "Html" => Some("text/html"),
            "plain" | "Plain" | "PLAIN" => Some("text/plain"),
            "word" | "WORD" | "Word" => {
                Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
            }
            _ => None,
        };
        match document_type_ {
            Some(type_) => {
                let mut file =
                    File::open(document_path).expect("Error opening the file path specified\n");
                let mut vec_of_bytes = Vec::new();
                file.read_to_end(&mut vec_of_bytes).unwrap();
                let build_blob = Blob::new(vec_of_bytes);
                let document_build = Document::builder()
                    .content(build_blob)
                    .content_type(type_)
                    .build();
                let outputs = client
                    .translate_document()
                    .source_language_code("auto")
                    .document(document_build)
                    .target_language_code(target_lang_code)
                    .send()
                    .await
                    .expect("Error while Translating Document\n");
                if let Some(translated_document) = outputs.translated_document {
                    if let Some(content) = translated_document.content {
                        let file_name = match document_type {
                            "html" | "HTML" | "Html" => Some(format!("TranslatedDocument.html")),
                            "plain" | "Plain" | "PLAIN" => Some(format!("TranslatedDocument.txt")),
                            "word" | "WORD" | "Word" => Some(format!("TranslatedDocument.docx")),
                            _ => None,
                        };
                        match file_name {
                            Some(file_name) => {
                                let mut file = OpenOptions::new()
                                    .create(true)
                                    .read(true)
                                    .write(true)
                                    .open(&file_name)
                                    .expect("Error while creating file\n");
                                let translated_content = content.into_inner();
                                file.write_all(&translated_content).unwrap();
                                match File::open(&file_name) {
                    Ok(_) => println!("The translated text for the given input is saved in the current directory under the filename '{}'\n",file_name.green().bold()),
                    Err(_) => println!("{}\n","Error while opening the file".red().bold()),
                }
                            }
                            None => {}
                        }
                    }
                }
            }
            None => {
                println!(
                    "Unsupported Document Type: {}\n",
                    document_type.red().bold()
                );
                println!("{}", "Supported Document Types are,".yellow().bold());
                println!("{}", "1) html or HTML or Html".yellow().bold());
                println!("{}", "2) plain or Plain or PLAIN".yellow().bold());
                println!("{}\n", "3) word or WORD or Word".yellow().bold());
            }
        }
    }
    pub async fn start_text_translation_job(
        &self,
        job_name: &str,
        target_lang_codes: Option<Vec<String>>,
        input_s3_uri: &str,
        document_type: &str,
        output_s3_uri: &str,
        role_arn: &str,
    ) {
        let client = TranslateClient::new(self.config);
        let document_type_ = match document_type {
            "html" | "HTML" | "Html" => Some("text/html"),
            "plain" | "Plain" | "PLAIN" => Some("text/plain"),
            "word" | "WORD" | "Word" => {
                Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
            }
            "ppt" | "PPT" | "Ppt" => {
                Some("application/vnd.openxmlformats-officedocument.presentationml.presentation")
            }
            "xlsx" | "XLSX" | "Xlsx" => {
                Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            }
            "xlf" | "XLF" | "Xlf" => Some("application/x-xliff+xml"),
            _ => None,
        };
        match document_type_ {
            Some(document_type) => {
                let client_token = match std::env::var("aws_access_key_id") {
                    Ok(access_key) => access_key,
                    Err(_) => {
                        println!("The '{}' environment variable is not found, so I am attempting to retrieve the access key from the credentials file","aws_access_key_id\n".yellow().bold());
                        let shared_credential = self.config.credentials_provider().unwrap();
                        let credentials = shared_credential.provide_credentials().await.unwrap();
                        credentials.access_key_id().to_string()
                    }
                };
                let input_config_build = InputDataConfig::builder()
                    .s3_uri(input_s3_uri)
                    .content_type(document_type)
                    .build();
                let output_config = OutputDataConfig::builder().s3_uri(output_s3_uri).build();
                let outputs = client
                    .start_text_translation_job()
                    .client_token(client_token)
                    .data_access_role_arn(role_arn)
                    .source_language_code("auto")
                    .set_target_language_codes(target_lang_codes)
                    .input_data_config(input_config_build)
                    .output_data_config(output_config)
                    .job_name(job_name)
                    .send()
                    .await
                    .expect("Error while Starting Text Transcription Job\n");
                let file_name = format!("TranscriptionJobIDFor{}.txt", job_name);
                let mut file = OpenOptions::new()
                    .create(true)
                    .read(true)
                    .write(true)
                    .open(&file_name)
                    .expect("Error while creating file\n");
                if let Some(job_id) = outputs.job_id {
                    println!("The Job ID: {}", job_id.green().bold());
                    let buf = format!("Transcription Job ID for the Job Name: {}", job_id);
                    file.write_all(buf.as_bytes()).unwrap();
                    match File::open(&file_name) {
                        Ok(_) => {
                            println!("The job ID has been written to a file named '{}' in the current directory for the job named '{}'",file_name.green().bold(),job_name.green().bold());
                            println!("{}\n","The job ID is necessary to retrieve details about the job in other options".yellow().bold());
                        }
                        Err(_) => println!("{}\n", "Error while writing the job id".red().bold()),
                    }
                }
                if let Some(job_status) = outputs.job_status {
                    let status = job_status.as_str();
                    println!(
                        "The Current Status of Transcipion Job Is: {}",
                        status.green().bold()
                    );
                }
                println!("{}\n","To check the status and obtain additional information about the transcription job, use the ---Describe Text Translation Job--- option".yellow().bold());
            }
            None => {
                println!(
                    "Unsupported Document Type: {}\n",
                    document_type.red().bold()
                );
                println!("{}\n", "Supported Document Types are".yellow().bold());
                println!("{}", "1) html or HTML or Html".yellow().bold());
                println!("{}", "2) plain or Plain or PLAIN".yellow().bold());
                println!("{}", "3) word or WORD or Word".yellow().bold());
                println!("{}", "4) ppt or PPT or Ppt".yellow().bold());
                println!("{}", "5) xlsx or XLSX or Xlsx ".yellow().bold());
                println!("{}\n", "6) xlf or XLF or Xlf".yellow().bold());
            }
        };
    }

    pub async fn describe_text_translation_job(&self, job_id: &str) {
        let client = TranslateClient::new(self.config);
        let outputs = client
            .describe_text_translation_job()
            .job_id(job_id)
            .send()
            .await
            .expect("Error While Describing Text Translation Job\n");
        if let Some(details) = outputs.text_translation_job_properties {
            if let Some(data_role_access_arn) = details.data_access_role_arn {
                println!(
                    "Data Access Role Amazon Resource Name(ARN): {}",
                    data_role_access_arn.green().bold()
                );
            }
            if let Some(job_name) = details.job_name {
                println!("Job Name: {}", job_name.green().bold());
            }
            if let Some(job_id) = details.job_id {
                println!("Job ID: {}", job_id.green().bold());
            }
            if let Some(source_language_code) = details.source_language_code {
                println!(
                    "Source Language Code: {}",
                    source_language_code.green().bold()
                );
                println!("{}\n","The source language of the document is automatically identified using the ;DetectDominantLanguages' API of the Comprehend service".yellow().bold());
            }
            if let Some(target_languages_codes) = details.target_language_codes {
                println!("{}\n", "Target Language Codes".yellow().bold());
                target_languages_codes.into_iter().for_each(|lang_code| {
                    println!("Language Code: {}", lang_code.green().bold());
                });
                println!("{}\n","Execute the 'Get Language Info' option to determine the language associated with the provided language codes".yellow().bold());
            }
            if let Some(job_details) = details.job_details {
                if let Some(document_count) = job_details.input_documents_count {
                    println!(
                        "Input Document Count: {}",
                        document_count.to_string().green().bold()
                    );
                }
                if let Some(success_counts) = job_details.translated_documents_count {
                    println!(
                        "Document Success Count: {}",
                        success_counts.to_string().green().bold()
                    );
                }
                if let Some(error_counts) = job_details.documents_with_errors_count {
                    println!(
                        "Document Error Count: {}",
                        error_counts.to_string().green().bold()
                    );
                }
            }
            if let Some(output_config) = details.output_data_config {
                if let Some(s3_uri) = output_config.s3_uri {
                    println!("Output S3 Uri: {}", s3_uri.green().bold());
                }
            }
            if let Some(submit_time) = details.submitted_time {
                let submittime = submit_time.fmt(DateTimeFormat::HttpDate).ok();
                if let Some(time) = submittime {
                    println!("Submitted Date and Time: {}", time.green().bold());
                }
            }
            if let Some(end_time) = details.end_time {
                let convert_time = end_time.fmt(DateTimeFormat::HttpDate).ok();
                if let Some(time) = convert_time {
                    println!("Finished Date and Time: {}", time.green().bold());
                }
            }
            if let Some(job_status) = details.job_status {
                let status = job_status.as_str();
                println!("Job Status: {}", status.green().bold());
            }
            if let Some(message) = details.message {
                println!("Message: {}", message.green().bold());
            }
            println!("");
        }
    }
    pub async fn list_translation_jobs(&self) {
        let client = TranslateClient::new(self.config);
        let outputs = client
            .list_text_translation_jobs()
            .send()
            .await
            .expect("Error while listing Translation Jobs\n");
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("ListTranslationJobsDetails.txt")
            .expect("Error while creating file\n");
        if let Some(transription_job_properties) = outputs.text_translation_job_properties_list {
            transription_job_properties.into_iter().for_each(|details| {
                if let Some(data_role_access_arn) = details.data_access_role_arn {
                    let buf = format!("Data Access Role Arn: {}\n", data_role_access_arn);
                    file.write_all(buf.as_bytes()).unwrap();
                }
                if let Some(job_name) = details.job_name {
                    let buf = format!("Job Name: {}\n", job_name);
                    file.write_all(buf.as_bytes()).unwrap();
                }
                if let Some(job_id) = details.job_id {
                    let buf = format!("Job ID: {}\n", job_id);
                    file.write_all(buf.as_bytes()).unwrap();
                }
                if let Some(submit_time) = details.submitted_time {
                    let convert_time = submit_time.fmt(DateTimeFormat::HttpDate).ok();
                    if let Some(time) = convert_time {
                        let buf = format!("Submitted Time: {}\n", time);
                        file.write_all(buf.as_bytes()).unwrap();
                    }
                }
                if let Some(end_time) = details.end_time {
                    let convert_time = end_time.fmt(DateTimeFormat::HttpDate).ok();
                    if let Some(time) = convert_time {
                        let buf = format!("Finished Time: {}\n", time);
                        file.write_all(buf.as_bytes()).unwrap();
                    }
                }
                if let Some(source_language_code) = details.source_language_code {
                    let buf = format!("Source Language Code: {}\n", source_language_code);
                    file.write_all(buf.as_bytes()).unwrap();
                }
                if let Some(target_languages_codes) = details.target_language_codes {
                    file.write_all("Target Language Codes:\n".as_bytes())
                        .unwrap();
                    target_languages_codes.into_iter().for_each(|lang_code| {
                        file.write_all(lang_code.as_bytes()).unwrap();
                        file.write_all("\n".as_bytes()).unwrap();
                    });
                }
                if let Some(output_config) = details.output_data_config {
                    if let Some(s3_uri) = output_config.s3_uri {
                        let buf = format!("Output S3 Uri: {}\n", s3_uri);
                        file.write_all(buf.as_bytes()).unwrap();
                    }
                }
                if let Some(job_details) = details.job_details {
                    if let Some(document_count) = job_details.input_documents_count {
                        let buf = format!("Input Document Counts: {}\n", document_count);
                        file.write_all(buf.as_bytes()).unwrap();
                    }
                    if let Some(success_counts) = job_details.translated_documents_count {
                        let buf = format!("Document Success Count: {}\n", success_counts);
                        file.write_all(buf.as_bytes()).unwrap();
                    }
                    if let Some(error_counts) = job_details.documents_with_errors_count {
                        let buf = format!("Document Error Count: {}\n", error_counts);
                        file.write_all(buf.as_bytes()).unwrap();
                    }
                }
                if let Some(job_status) = details.job_status {
                    let status = job_status.as_str();
                    let buf = format!("Job Status: {}\n", status);
                    file.write_all(buf.as_bytes()).unwrap();
                }
                if let Some(message) = details.message {
                    let buf = format!("Job Message: {}\n\n\n", message);
                    file.write_all(buf.as_bytes()).unwrap();
                }
            });
        }
        match File::open("ListTranslationJobsDetails.txt") {
            Ok(_) => {
                println!("The details of the translation job list have been written to the current directory with the filename '{}'\n","ListTranslationJobsDetails.txt".green().bold());
                println!("{}\n","If you want to access specific job information without leaving the terminal, execute the ---Describe Text Translation Job--- option to learn more about the job details".yellow().bold());
            }
            Err(_) => println!("{}\n", "Error writing File".red().bold()),
        }
    }
}
