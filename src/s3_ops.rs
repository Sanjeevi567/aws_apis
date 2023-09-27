use aws_config::SdkConfig;
use aws_sdk_s3::{
    presigning::PresigningConfig,
    primitives::ByteStream,
    types::{
        BucketLocationConstraint, CompletedMultipartUpload, CompletedPart,
        CreateBucketConfiguration, ObjectCannedAcl,
    },
    Client as S3Client,
};
use colored::Colorize;
use dotenv::dotenv;
use regex::Regex;
use std::{
    env::var,
    fs::{File, OpenOptions},
    io::Write,
    time::{Duration, SystemTime},
};
/// The core structure for performing operations on the [`S3 client`](https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/struct.Client.html) eliminates the need for
/// API users to provide credentials each time they use the service. Instead,
/// these credentials are abstracted by this structure and its inherent functions
/// and methods
#[derive(Debug)]
pub struct S3Ops {
    config: SdkConfig,
}
impl S3Ops {
    /// This is a private function used internally to verify service credentials.
    /// By doing so, users of the API are spared from having to repeatedly specify
    /// their credentials whenever they use the service
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }

    /// This function accepts an [`SdkConfig`](https://docs.rs/aws-config/latest/aws_config/struct.SdkConfig.html), retrieves the region name from it if
    /// available; otherwise, it sets it to an empty string and then constructs a S3Ops instance   
    pub fn build(config: SdkConfig) -> Self {
        Self { config: config }
    }

    ///Create a new bucket in your AWS account and ensure you specify the region
    /// name; otherwise, you may receive a panic message from AWS APIs
    pub async fn create_bucket(&self, bucket_name: &str) {
        dotenv().ok();
        let config = self.get_config();
        let client = S3Client::new(config);
        let region_name = match self.get_config().region(){
             Some(region) => region.to_string(),
             None => var("REGION").unwrap_or("The region value is read from the .env file in the current directory if it is not provided in the credential file".into())
        };
        let constraint = BucketLocationConstraint::from(region_name.as_str());
        let location = CreateBucketConfiguration::builder()
            .location_constraint(constraint)
            .build();

        let client = client
            .create_bucket()
            .bucket(bucket_name)
            .create_bucket_configuration(location);
        let colored_msg = "Error from create_bucket function".red().bold();
        client
            .send()
            .await
            .map(|_| {
                let colored_bucket = bucket_name.green().bold();
                println!(
                    "Congratulations! The bucket with the name {colored_bucket} has been successfully created"
                );
            })
            .expect(&colored_msg);
    }

    /// Return the available buckets in your account as a vector of strings
    pub async fn get_buckets(&self) -> Vec<String> {
        let config = self.get_config();
        let client = S3Client::new(config);

        let mut bucket_lists = Vec::new();
        let colored_msg = "Error from get_buckets function".red().bold();
        let output = client.list_buckets().send().await.expect(&colored_msg);
        let bucket_list = output.buckets();

        if let Some(bucket_names) = bucket_list {
            bucket_names.iter().for_each(|bucket| {
                bucket_lists.push(bucket.name().unwrap().to_string());
            })
        }

        bucket_lists
    }

    /// Delete the bucket from your AWS services if the specified bucket is
    ///  available and the credentials have the necessary rights.
    pub async fn delete_bucket(&self, bucket_name: &str) {
        let config = self.get_config();
        let client = S3Client::new(config);

        let client = client.delete_bucket().bucket(bucket_name);
        let colored_msg = "Error from delete_bucket function".red().bold();
        client.send().await.expect(&colored_msg);
        let current_buckets = self.get_buckets().await;
        println!("Currently available buckets in your aws account\n");
        current_buckets.into_iter().for_each(|bucket| {
            let bucket = bucket.green().bold();
            println!("{bucket}\n");
        })
    }

    ///These methods work on Ubuntu but not on Windows due to differences in stack size. In Ubuntu, the stack size is larger than in Windows, which is why it causes a stack overflow in Windows. As a result, I tested these methods on Ubuntu successfully but encountered a stack overflow issue on Windows.
    ///I attempted to use these methods in a different thread with a stack size of (32*1024*1024), but it still resulted in a stack overflow
    /// Ensure successful execution in release mode by using cargo build --release.
    /// If you have any solutions or suggestions to address this issue in build time, please let
    /// me know by leaving a comment [`here`]()

    /// Retrieve the objects/keys from a specified bucket.
    pub async fn retrieve_keys_in_a_bucket(&self, bucket_name: &str) -> Vec<String> {
        let config = self.get_config();
        let client = S3Client::new(config);

        let mut objects_in_bucket = Vec::new();
        let outputs = client
            .list_objects_v2()
            .bucket(bucket_name)
            .send()
            .await
            .expect("Error While Retrieving keys in a Bucket\n");

        if let Some(keys) = outputs.contents {
            keys.into_iter().for_each(|object| {
                if let Some(key) = object.key {
                    objects_in_bucket.push(key);
                }
            })
        }
        objects_in_bucket
    }
    pub async fn list_objects_given_prefix(
        &self,
        bucket_name: &str,
        path_prefix: &str,
    ) -> Vec<String> {
        let config = self.get_config();
        let client = S3Client::new(config);
        let mut keys_in_the_prefix = Vec::new();
        let outputs = client
            .list_objects_v2()
            .prefix(path_prefix)
            .bucket(bucket_name)
            .send()
            .await
            .expect("Error while listing objects give path prefix\n");
        if let Some(keys) = outputs.contents {
            keys.into_iter().for_each(|object_key| {
                let key = object_key.key;
                if let Some(key_) = key {
                    keys_in_the_prefix.push(key_);
                }
            });
        }
        keys_in_the_prefix
    }

    /// Store the content in the S3
    ///  storage with the specified bucket name (which should already exist),
    /// key name (to retrieve data later), and path to the data.
    pub async fn upload_content_to_a_bucket(
        &self,
        bucket_name: &str,
        data_path: &str,
        name_of_object: &str,
    ) {
        let config = self.get_config();
        let client = S3Client::new(config);

        let build_body_data = ByteStream::read_from()
            .path(data_path)
            .build()
            .await
            .unwrap();
        let colored_msg = "Error from upload_content_to_a_bucket function"
            .red()
            .bold();
        client
            .put_object()
            .bucket(bucket_name)
            .key(name_of_object)
            .body(build_body_data)
            .send()
            .await
            .expect(&colored_msg);
        println!(
            "The provided object {} has been successfully updated in the bucket {}\n",
            data_path.green().bold(),
            bucket_name.green().bold()
        );

        /*
        let current_objects = self.retrieve_keys_in_a_bucket(bucket_name).await;
        println!("Currently available keys/objects in your {bucket_name} bucket\n");
        current_objects.into_iter().for_each(|key| {
            let key = key.green().bold();
            println!("{key}\n");
        }); */
    }
    pub async fn put_object_acl(
        &self,
        bucket_name: &str,
        name_of_object: &str,
        acl_permission: &str,
    ) {
        let config = self.get_config();
        let client = S3Client::new(config);
        let acl_permission_build = ObjectCannedAcl::from(acl_permission);
        let acl_permission_str = acl_permission_build.as_str().to_owned();
        client
            .put_object_acl()
            .key(name_of_object)
            .bucket(bucket_name)
            .acl(acl_permission_build)
            .send()
            .await
            .expect("Error while putting ACL on bucket\n");
        println!(
            "The ACL permission '{}' has been successfully applied to the object key '{}' within the '{}' bucket\n",
            acl_permission_str.green().bold(),
            name_of_object.green().bold(),
            bucket_name.green().bold()
        );
        match acl_permission {
            "public-read" => {
                let msg =format!("The permission is set to '{}' making the object accessible to anyone who has the object's URL",acl_permission_str);
                println!("{}\n", msg.green().bold());
            }
            "public-read-write" => {
                let msg = format!("The permission is configured as '{}' granting read and write access to anyone who accesses the object's URL",acl_permission_str);
                println!("{}\n", msg.green().bold());
            }
            _ => {}
        }
    }

    ///Upload large files using chunks instead of uploading the entire file, while
    /// accepting the same parameters as the method above.
    pub async fn mulitpart_upload(&self, bucket_name: &str, object_name: &str, data_path: &str) {
        let config = self.get_config();
        let client = S3Client::new(config);

        let colored_msg = "Error from multipart_upload function".red().bold();
        let mulit_part = client
            .create_multipart_upload()
            .bucket(bucket_name)
            .key(object_name)
            .send()
            .await
            .expect(&colored_msg);

        let data = ByteStream::from_path(data_path).await.unwrap();

        let upload_id = mulit_part.upload_id().unwrap();

        let upload_part_result = client
            .upload_part()
            .bucket(mulit_part.bucket().unwrap())
            .key(mulit_part.key().unwrap())
            .body(data)
            .upload_id(upload_id)
            .part_number(30)
            .send()
            .await
            .unwrap();

        let mut completed_part = Vec::new();
        completed_part.push(
            CompletedPart::builder()
                .e_tag(upload_part_result.e_tag().unwrap_or_default())
                .part_number(30)
                .build(),
        );

        let completed_multipart = CompletedMultipartUpload::builder()
            .set_parts(Some(completed_part))
            .build();

        client
            .complete_multipart_upload()
            .bucket(bucket_name)
            .key(object_name)
            .multipart_upload(completed_multipart)
            .upload_id(upload_id)
            .send()
            .await
            .unwrap();
    }
    /// Download the content to the current directory, using the name of the content
    /// file being downloaded. This process accepts a bucket name and key to retrieve
    /// the actual data
    pub async fn download_content_from_bcuket(
        &self,
        bucket_name: &str,
        object_name: &str,
        path_prefix: Option<&str>,
        print_info: bool,
    ) {
        let config = self.get_config();
        let client = S3Client::new(config);

        let client = client.get_object().bucket(bucket_name).key(object_name);

        let colored_msg = "Error from download_content_from_bucket function"
            .red()
            .bold();
        let get_body_data = client.send().await.expect(&colored_msg);

        //The regex engine doesn't support look-arounds, including look-aheads and look-behinds. Therefore,
        //this option is used as a secondary condition. This ensures that even if it matches the dot, it won't
        // have a chance to retrieve values with dots, as the first match takes precedence.
        let have_slash_and_dot_pattern =
            Regex::new(r#"([^./]+)\.([^/]+)"#).expect("Error while parsing Regex Syntax\n");
        let have_slash_but_no_extension_pattern =
            Regex::new(r#"/([^/]+)$"#).expect("Error while parsing regex syntax");
        let no_slash_no_dot_pattern =
            Regex::new(r#"^[^./]*$"#).expect("Error while parsing Regex syntax");
        let file_name: Vec<String> = if have_slash_and_dot_pattern.is_match(object_name) {
            have_slash_and_dot_pattern
                .find_iter(object_name)
                .map(|string| string.as_str().to_string())
                .collect()
        } else if have_slash_but_no_extension_pattern.is_match(object_name) {
            have_slash_but_no_extension_pattern
                .find_iter(object_name)
                .map(|string| string.as_str().to_string())
                .collect()
        } else {
            no_slash_no_dot_pattern
                .find_iter(object_name)
                .map(|string| string.as_str().to_string())
                .collect()
        };
        let mut file_name = file_name.join("");
        if file_name.starts_with("/") {
            file_name.remove(0);
        };
        if print_info {
            let content_type = get_body_data.content_type().unwrap().green().bold();
            println!("Content type of response body: {}", content_type);

            let content_length = get_body_data.content_length() as f64 * 0.000001;
            let content_length_colored = content_length.to_string().green().bold();
            println!("The content length/size of data in MB: {content_length_colored:.3}mb");
            let last_modified = get_body_data
                .last_modified
                .map(|format| {
                    format
                        .fmt(aws_sdk_memorydb::primitives::DateTimeFormat::HttpDate)
                        .ok()
                })
                .flatten();
            if let Some(time) = last_modified {
                println!("Last Modified: {}\n", time.green().bold());
            }
        }
        let mut file = match path_prefix {
            Some(prefix) => {
                let file_path = format!("{prefix}{file_name}");
                OpenOptions::new()
                    .create(true)
                    .read(true)
                    .write(true)
                    .open(file_path)
                    .expect(
                        "Error while creating file In Downloading Content from Bucket Function\n",
                    )
            }
            None => File::create(file_name).expect("Error while creating file\n"),
        };
        let bytes = get_body_data.body.collect().await.unwrap();
        let bytes = bytes.into_bytes();
        match file.write_all(&*bytes) {
            Ok(_) => {
                if print_info {
                    println!("{}\n", "Writing data...".bright_green().bold());
                    let colored_key_name = object_name.green().bold();
                    println!(
                        "The content of the {colored_key_name} is saved in the current directory\n"
                    );
                }
            }

            Err(_) => {
                if print_info {
                    println!("{}\n", "Error while writing\n".red().bold());
                }
            }
        }
    }

    pub async fn get_presigned_url_for_an_object(
        &self,
        bucket_name: &str,
        object_name: &str,
        end_time: u64,
    ) {
        use chrono::prelude::*;
        use fast_qr::convert::{image::ImageBuilder, Builder, Shape};
        use fast_qr::qr::QRBuilder;

        let config = self.get_config();
        let client = S3Client::new(config);

        let start_time = SystemTime::now();
        let utc: DateTime<Utc> = Utc::now();

        //converting to seconds from hour given by end_time
        let hour_to_secs = end_time * 60 * 60;
        let expire_time = Duration::from_secs(hour_to_secs);

        let presigning_config = PresigningConfig::builder()
            .start_time(start_time)
            .expires_in(expire_time)
            .build()
            .unwrap();

        let expired_in = presigning_config.expires();

        let get_hour = (60 * 60 * end_time) / expired_in.as_secs();

        let colored_msg = "Error from get_presigned_url_for_an_object".red().bold();
        let presigned_req = client
            .get_object()
            .bucket(bucket_name)
            .key(object_name)
            .presigned(presigning_config)
            .await;

        let presigned_info = presigned_req.expect(&colored_msg);

        let method_of_content = presigned_info.method().as_str();
        let colored_method = method_of_content.green().bold();
        println!("http method of the content: {}\n", colored_method);

        let content_url = presigned_info.uri().to_string();
        let colored_uri = content_url.blue().bold();
        let colored_end_time = end_time.to_string().green().bold();
        println!(
            "The URI for the content is: {}\n and the expiration time is: {} hour from now\n",
            colored_uri, colored_end_time
        );
        println!("{}\n","Press and hold Ctrl while clicking the link to open it, and it will automatically begin downloading".green().bold());
        println!(
            "{}\n",
            "Visit https://tinyurl.com/app to shorten your URL"
                .blue()
                .bold()
        );

        // Generating text file
        let mut file = File::create("./uri.txt").unwrap();
        let year = utc.year();
        let month = utc.month();
        let week_day = utc.weekday();
        let day = utc.day();
        let minute = utc.minute();
        let hour = utc.hour();
        let secs = utc.second();
        let format_string_to_write_into = 
        format!("The URL for the content is: {content_url}\n\n\nStarted at\nyear: {year}\nmonth: {month}\nweek_day: {week_day}\nday: {day}\nminutes: {minute}\nhours: {hour}\nseconds: {secs}\n\nExpired at: {get_hour} h");
        file.write_all(format_string_to_write_into.as_bytes())
            .unwrap();
        println!(
            "{}\n",
            r#"The content has been written to "uri.txt" in the current directory."#
                .green()
                .bold()
        );

        //generating qr image for the uri
        let qrcode = QRBuilder::new(content_url.as_str()).build().unwrap();
        ImageBuilder::default()
            .shape(Shape::Square)
            .background_color([255, 255, 255, 0])
            .fit_width(600)
            .fit_height(600)
            .to_file(&qrcode, "./uri_qr.png")
            .unwrap();

        println!("{}\n","A QR code has been generated for the content's URL and is saved in the current directory as 'uri_qr.png'".green().bold());
    }

    /// Delete the content or key in the provided bucket. Please be cautious, as
    /// this action will permanently remove the content from the service
    pub async fn delete_content_in_a_bucket(&self, bucket_name: &str, object_name: &str) {
        let config = self.get_config();
        let client = S3Client::new(config);
        let colored_msg = "Error from delete_content_in_a_bucket function"
            .red()
            .bold();
        client
            .delete_object()
            .bucket(bucket_name)
            .key(object_name)
            .send()
            .await
            .map(|_| {
           let colored_key_name = object_name.red().bold();
           let colored_bucket_name = bucket_name.red().bold();
            println!( "The object {colored_key_name} in bucket {colored_bucket_name} has been deleted");
            })
            .expect(&colored_msg);

        let current_objects = self.retrieve_keys_in_a_bucket(bucket_name).await;
        println!("Currently available keys/objects in your {bucket_name} bucket\n");
        current_objects.into_iter().for_each(|key| {
            let key = key.green().bold();
            println!("{key}\n");
        });
    }
}
