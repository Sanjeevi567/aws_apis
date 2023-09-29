use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

use aws_config::SdkConfig;
use aws_sdk_pinpoint::primitives::Blob;
use aws_sdk_rekognition::{
    operation::{
        get_face_detection::GetFaceDetectionOutput,
        get_face_liveness_session_results::GetFaceLivenessSessionResultsOutput,
        get_text_detection::GetTextDetectionOutput,
    },
    types::{
        Attribute, BoundingBox, CreateFaceLivenessSessionRequestSettings, FaceDetail,
        FaceDetection, Image, LivenessOutputConfig, S3Object, TextDetection, TextDetectionResult,
        Video,
    },
    Client as RekogClient,
};
use colored::Colorize;
use std::ops::Deref;

use crate::{
    create_celebrity_pdf,
    pdf_writer::{create_face_result_pdf, create_text_result_pdf},
};

pub struct RekognitionOps {
    config: SdkConfig,
}
impl RekognitionOps {
    pub fn build(config: SdkConfig) -> Self {
        Self { config }
    }
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }
    /// [`Attribute`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/enum.Attribute.html)
    pub async fn detect_faces(&self, key_name: &str, bucket_name: &str) -> Vec<FaceDetails> {
        let config = self.get_config();
        let client = RekogClient::new(config);
        let s3_object_builder = S3Object::builder()
            .name(key_name)
            .bucket(bucket_name)
            .build();
        //  https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/operation/detect_faces/builders/struct.DetectFacesFluentBuilder.html#method.set_attributes
        let attribute = vec![
            Attribute::AgeRange,
            Attribute::Gender,
            Attribute::Smile,
            Attribute::Beard,
            Attribute::Default,
        ];
        let image_builder = Image::builder().s3_object(s3_object_builder).build();
        let detect_face_output = client
            .detect_faces()
            .set_attributes(Some(attribute))
            .image(image_builder)
            .send()
            .await
            .expect("Error while detecting faces\n");
        let mut vec_of_facedetails = Vec::new();

        if let Some(face_detail) = detect_face_output.face_details {
            face_detail.into_iter().for_each(|outputs| {
                vec_of_facedetails.push(FaceDetails(outputs));
            });
        }
        vec_of_facedetails
    }

    pub async fn create_collection(&self, collection_id: &str) {
        let config = self.get_config();
        let client = RekogClient::new(config);
        let outputs = client
            .create_collection()
            .collection_id(collection_id)
            .send()
            .await
            .expect("Error while creating Create Collection\n");
        if let Some(arn) = outputs.collection_arn {
            println!("Collection Arn: {}", arn.green().bold());
        }
        if let Some(model_version) = outputs.face_model_version {
            println!("Model Version: {}", model_version.green().bold());
        }
        if let Some(code) = outputs.status_code {
            let format = format!("Status Code: {}\n", code);
            println!("{}\n", format.green().bold());
        }
    }
    pub async fn index_faces(&self, bucket_name: &str, key_image_name: &str, collection_id: &str) {
        let config = self.get_config();
        let client = RekogClient::new(config);
        let s3_object_builder = S3Object::builder()
            .bucket(bucket_name)
            .name(key_image_name)
            .build();
        let image_builder = Image::builder().s3_object(s3_object_builder).build();
        let outputs = client
            .index_faces()
            .collection_id(collection_id)
            .image(image_builder)
            .send()
            .await
            .expect("Error while Indexing Faces\n");
        if let Some(face_record) = outputs.face_records {
            face_record.into_iter().for_each(|face| {
                if let Some(face) = face.face {
                    if let Some(face_id) = face.face_id {
                        println!("Face Id For the Uploaded Face: {}\n", face_id.green().bold());
                    }
                }
            })
        }
    }
    pub async fn search_faces(&self, collection_id: &str, face_id: &str) {
        let config = self.get_config();
        let client = RekogClient::new(config);
        let outputs = client
            .search_faces()
            .collection_id(collection_id)
            .face_id(face_id)
            .send()
            .await
            .expect("Error While Searching Faces\n");
        if let Some(search_face_output) = outputs.face_matches {
            search_face_output.into_iter().for_each(|similarity| {
                if let Some(simiarity_) = similarity.similarity {
                    println!("Similarity With Already Indexed Face: {}\n", simiarity_);
                }
            })
        }
    }
    pub async fn detect_texts(&self, bucket_name: &str, key_name: &str) -> Vec<TextDetect> {
        let config = self.get_config();
        let client = RekogClient::new(config);

        let s3_object_builder = S3Object::builder()
            .bucket(bucket_name)
            .name(key_name)
            .build();

        let image_builder = Image::builder().s3_object(s3_object_builder).build();

        let detect_text_output = client
            .detect_text()
            .image(image_builder)
            .send()
            .await
            .expect("Error while detetecting text\n");

        let mut vec_of_text_detect = Vec::new();
        if let Some(text_detection) = detect_text_output.text_detections {
            text_detection.into_iter().for_each(|texts| {
                vec_of_text_detect.push(TextDetect(texts));
            });
        }
        vec_of_text_detect
    }
    pub async fn start_text_detection_task(
        &self,
        bucket_name: &str,
        key_video_name: &str,
    ) -> Option<String> {
        let config = self.get_config();
        let client = RekogClient::new(config);

        let s3_object_builder = S3Object::builder()
            .bucket(bucket_name)
            .name(key_video_name)
            .build();
        let video_builder = Video::builder().s3_object(s3_object_builder).build();

        let output = client
            .start_text_detection()
            .video(video_builder)
            .send()
            .await
            .expect("Error while start text detection task\n");
        let job_id = output.job_id;

        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("start_text_detection_job_id.txt")
            .expect("Error while creating job id file");
        if let Some(id) = job_id.as_deref() {
            println!(
                "The Job ID for the start text detection task is: {}\n",
                id.green().bold()
            );
            let id = format!("The Job ID for initiating text detection is: {}", id);
            match file.write_all(id.as_bytes()) {
                Ok(_) => {
                    println!(
                        "{}\n",
                        "The job ID has been successfully written to the current directory"
                            .green()
                            .bold()
                    );
                    println!("{}","Before initiating a new text detection task, please change the job ID file name in the current directory where the CLI is running".yellow().bold());
                    println!(
                        "{}\n",
                        "This ensures that the old file is not replaced by the new job ID"
                            .yellow()
                            .bold()
                    );
                }
                Err(_) => println!("Error while writing job id...\n"),
            }
        }

        job_id
    }
    pub async fn get_text_detection_results(&self, text_job_id: &str) -> GetTextInfo {
        let config = self.get_config();
        let client = RekogClient::new(config);
        let get_text_detection_ouput = client
            .get_text_detection()
            .job_id(text_job_id)
            .send()
            .await
            .expect("Error while getting text detection\n");
        GetTextInfo(get_text_detection_ouput)
    }
    pub async fn start_face_detection_task(
        &self,
        bucket_name: &str,
        key_video_name: &str,
    ) -> Option<String> {
        let config = self.get_config();
        let client = RekogClient::new(config);

        let s3_object_builder = S3Object::builder()
            .bucket(bucket_name)
            .name(key_video_name)
            .build();

        let video_builder = Video::builder().s3_object(s3_object_builder).build();

        let output = client
            .start_face_detection()
            .video(video_builder)
            .send()
            .await
            .expect("Error while starting face detetction task\n");

        let job_id = output.job_id;

        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("start_face_detection_job_id.txt")
            .expect("Error while creating job id file");
        if let Some(id) = job_id.as_deref() {
            println!(
                "The job ID for the start face detection task is: {}\n",
                id.green().bold()
            );
            let id = format!("The Job ID for initiating face detection is: {}", id);
            match file.write_all(id.as_bytes()) {
                Ok(_) => {
                    println!(
                        "{}\n",
                        "The job ID has been successfully written to the current directory"
                            .green()
                            .bold()
                    );
                    println!("{}","Before initiating a new face detection task, please change the job ID file name in the current directory where the CLI is running".yellow().bold());
                    println!(
                        "{}\n",
                        "This ensures that the old file is not replaced by the new job ID"
                            .yellow()
                            .bold()
                    );
                }
                Err(_) => println!("Error while writing job id...\n"),
            }
        }

        job_id
    }

    pub async fn get_face_detection_results(&self, face_job_id: &str) -> GetFaceInfo {
        let config = self.get_config();
        let client = RekogClient::new(config);
        let get_face_detection_output = client
            .get_face_detection()
            .job_id(face_job_id)
            .send()
            .await
            .expect("Error while getting face detection result\n");

        GetFaceInfo(get_face_detection_output)
    }
    pub async fn recognize_celebrities(
        &self,
        local_image_path: Option<&str>,
        bucket_name: Option<&str>,
        image_key_name: Option<&str>,
    ) {
        let config = self.get_config();
        let client = RekogClient::new(config);
        let image = match local_image_path {
            Some(local_image_path_) => {
                let mut file = std::fs::File::open(local_image_path_)
                    .expect("Error while reading the path you specified\n");
                let mut vec_of_u8s = Vec::new();
                file.read_to_end(&mut vec_of_u8s).unwrap();
                let bytes_builder = Blob::new(vec_of_u8s);
                Image::builder().bytes(bytes_builder).build()
            }
            None => {
                let s3_object_builder = S3Object::builder()
                    .set_bucket(bucket_name.map(|to_str| to_str.to_string()))
                    .set_name(image_key_name.map(|to_str| to_str.to_string()))
                    .build();
                Image::builder().s3_object(s3_object_builder).build()
            }
        };

        let outputs = client
            .recognize_celebrities()
            .image(image)
            .send()
            .await
            .expect("Erro while Recognizing Celebrities\n");
        if let Some(celebrity_faces) = outputs.celebrity_faces {
            let headers = vec![
                "Celebrity Name".into(),
                "Unique Celebrity Identifier".into(),
                "Gender of the Celebrity".into(),
                "Face Location of the Celebrity's Face".into(),
                "Is the Celebrity Smiling?".into(),
            ];
            let mut records = Vec::new();
            let file_name = format!("CelebrityDetails.txt");
            let mut file = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(&file_name)
                .expect("Error while creating file\n");
            celebrity_faces.into_iter().for_each(|faces| {
                if let Some(name) = faces.name {
                    let famous_for = match name.as_str() {
                        "Ralph Fiennes" => format!("{name}(voldemort)"),
                        "Chris Hemsworth" => format!("{name}(Odin Son Thor)"),
                        "Tom Hiddleston" => format!("{name}(Loki)"),
                        "Johnny Depp" => format!("{name}(Captain Jack Sparrow)"),
                        "Tom Holland" => format!("{name}(Little Spider Man)"),
                        "Tobey Maguire" => format!("{name}(Amazing Spider Man)"),
                        "Andrew Garfield" => format!("{name}(Amazing Spider Man)"),
                        "Robert Downey Jr." => format!("{name}(Iron Man)"),
                        "Tom Felton" => format!("{name}(Draco Malfoy)"),
                        "Rupert Grint" => format!("{name}(Ron Weasley)"),
                        "Emma Watson" => format!("{name}(Hermione Granger)"),
                        "Daniel Radcliffe" => format!("{name}(Harry Potter)"),
                        "Mark Alan" => format!("{name}(Hulk)"),
                        "Chris Evans" => format!("{name}(Captain America)"),
                        "Chadwick Boseman" => format!("{name}(Black Panther)"),
                        "Vin Diesel" => format!("{name}(Dominic Toretto,I'm Groot)"),
                        "Paul Walker" => format!("{name}(Car Racer)"),
                        "Joseph Vijay" => format!("{name}(Ilaya Thalapathy)"),
                        "Rajinikanth" => format!("{name}(Super Star)"),
                        "Kamal Haasan" => format!("{name}(Ulaga Nayagan)"),
                        "Ajith Kumar" => format!("{name}(Ultimate Star)"),
                        _ => format!("{name}"),
                    };
                    println!("Celebrity Name: {}", famous_for.green().bold());
                    let buf = format!("Celebrity Name: {}\n", famous_for);
                    file.write_all(buf.as_bytes()).unwrap();
                    records.push(famous_for);
                }
                if let Some(id) = faces.id {
                    println!("Celebrity Amazon ID: {}", id.green().bold());
                    let buf = format!("Celebrity Amazon ID: {}\n", id);
                    file.write_all(buf.as_bytes()).unwrap();
                    records.push(id);
                }
                if let Some(gender) = faces.known_gender {
                    let gender_ = gender.r#type;
                    if let Some(genderr) = gender_ {
                        let finall = genderr.as_str().to_string();
                        println!("Celebrity Gender: {}", finall.green().bold());
                        let buf = format!("Celebrity Gender: {}", finall);
                        file.write_all(buf.as_bytes()).unwrap();
                        records.push(finall);
                    }
                }
                if let Some(face) = faces.face {
                    let mut bbox_string = String::new();
                    if let Some(bbox) = face.bounding_box {
                        if let (Some(width), Some(height), Some(left), Some(top)) =
                            (bbox.width, bbox.height, bbox.left, bbox.top)
                        {
                            let format_bbox =
                                format!("Width: {width:.2},Height: {height:.2},Left: {left:.2},Top: {top:.2}");
                            println!(
                                "Celebrity Face Location Info: {}",
                                format_bbox.green().bold()
                            );
                            let buf = format!("Bouding Box Details: {}\n", format_bbox);
                            file.write_all(buf.as_bytes()).unwrap();
                            bbox_string.push_str(&format_bbox);
                        }
                    }
                    records.push(bbox_string);
                    if let Some(smile) = face.smile {
                        let format_smile = format!("{}", smile.value);
                        println!(
                            "Is the Celebrity Smiling: {}\n",
                            format_smile.green().bold()
                        );
                        let buf = format!("Is the Celebrity Smiling: {}", format_smile);
                        file.write_all(buf.as_bytes()).unwrap();
                        records.push(format_smile);
                    }
                }
            });
            match std::fs::File::open(file_name) {
                Ok(_) => println!(
                    "{}\n",
                    "The text file has been successfully written to the current directory"
                        .green()
                        .bold()
                ),
                Err(_) => println!("{}\n", "Error while writing File".red().bold()),
            }
            create_celebrity_pdf(
                headers,
                records,
                local_image_path,
                (bucket_name, image_key_name),
            )
            .await;
        }
    }
    pub async fn create_face_liveness(&self, bucket_name: &str) {
        let config = self.get_config();
        let client = RekogClient::new(config);

        let s3_object_builder = LivenessOutputConfig::builder()
            .s3_bucket(bucket_name)
            .s3_key_prefix("faceliveness/")
            .build();

        let request_settings_builder = CreateFaceLivenessSessionRequestSettings::builder()
            .output_config(s3_object_builder)
            .build();

        let output = client
            .create_face_liveness_session()
            .settings(request_settings_builder)
            .send()
            .await
            .expect("Error while creating face liveness\n");

        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("session_id.txt")
            .expect("Error while creating file\n");
        if let Some(session_id) = output.session_id {
            let colored_session_id = session_id.green().bold();
            let buf = format!("The session identifier is: {session_id}\nThe key prefix for the S3 object is set to 'faceliveness/' in the bucket named {bucket_name}");
            println!("The session ID for the 'CreateFaceLiveness' project is: {colored_session_id}\nThe key prefix for the S3 object is set to 'faceliveness/' in the bucket named {bucket_name}\n");
            match file.write_all(buf.as_bytes()) {
                Ok(_) => println!(
                    "{}\n",
                    "Session ID has been written to the current directory"
                        .green()
                        .bold()
                ),
                Err(_) => println!(""),
            }
        }
    }
    pub async fn get_face_liveness_session_results(&self, session_id: &str) -> LivenessOutput {
        let config = self.get_config();
        let client = RekogClient::new(config);

        let get_faceliveness_session_results_ouput = client
            .get_face_liveness_session_results()
            .session_id(session_id)
            .send()
            .await
            .expect("Error while getting faceliveness results\n");
        LivenessOutput(get_faceliveness_session_results_ouput)
    }
}

pub struct LivenessOutput(GetFaceLivenessSessionResultsOutput);

impl LivenessOutput {
    pub fn get_liveness_status(&self) -> Option<&str> {
        self.0.status().map(|status| status.as_str())
    }
    pub fn get_confidence(&self) -> Option<f32> {
        self.0.confidence()
    }
    pub fn get_reference_image_type(&self) -> Option<Option<RefImageType>> {
        self.0.reference_image().map(|audit_image| {
            let s3_object = audit_image.s3_object();
            let bounding_box = audit_image.bounding_box();
            if let (Some(s3object), Some(bbox)) = (s3_object, bounding_box) {
                Some(RefImageType {
                    bucket_name: s3object.bucket.clone(),
                    key_name: s3object.name.clone(),
                    width: bbox.width,
                    height: bbox.height,
                    left: bbox.left,
                    top: bbox.top,
                })
            } else {
                None
            }
        })
    }
}
pub struct RefImageType {
    bucket_name: Option<String>,
    key_name: Option<String>,
    width: Option<f32>,
    height: Option<f32>,
    left: Option<f32>,
    top: Option<f32>,
}
impl RefImageType {
    pub fn get_s3_info(&self) -> (Option<&str>, Option<&str>) {
        (self.bucket_name.as_deref(), self.key_name.as_deref())
    }
    pub fn get_bounding_box_info(&self) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
        (self.width, self.height, self.left, self.top)
    }
}

/// [`TextDetection`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.TextDetection.html)
pub struct TextDetect(TextDetection);
impl TextDetect {
    pub fn build(type_: TextDetection) -> Self {
        Self(type_)
    }
    pub fn detected_text(&mut self) -> Option<String> {
        self.0.detected_text.take()
    }
    /// [`TextType`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/enum.TextTypes.html)
    pub fn text_type(&self) -> Option<String> {
        self.0
            .r#type()
            .map(|text_type| text_type.as_str().to_string())
    }
    pub fn confidence(&self) -> Option<f32> {
        self.0.confidence
    }
    ///[`Geometry`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Geometry.html)
    fn get_geometry_region(&self) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
        let region = if let Some(region_) = self.0.geometry.as_ref() {
            if let Some(bbox) = region_.bounding_box() {
                if let (Some(width), Some(height), Some(left), Some(top)) =
                    (bbox.width, bbox.height, bbox.left, bbox.top)
                {
                    (Some(width), Some(height), Some(left), Some(top))
                } else {
                    (None, None, None, None)
                }
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };
        region
    }
}
pub struct GetTextInfo(GetTextDetectionOutput);
impl GetTextInfo {
    pub fn job_status(&mut self) -> Option<String> {
        let job_status = if let Some(status) = self.0.job_status.take() {
            Some(status.as_str().to_string())
        } else {
            None
        };
        job_status
    }
    pub fn status_message(&mut self) -> Option<String> {
        self.0.status_message.take()
    }
    fn s3_details(&mut self) -> (String, String) {
        let mut bucket_name = String::new();
        let mut video_name = String::new();
        self.0.video.take().map(|video| {
            if let Some(s3_object) = video.s3_object {
                if let Some(bucket_name_) = s3_object.bucket() {
                    bucket_name.push_str(bucket_name_);
                }
                if let Some(key_name_) = s3_object.name() {
                    video_name.push_str(key_name_);
                }
            }
        });
        (bucket_name, video_name)
    }
    fn text_detect_result(&mut self) -> Vec<TextDetectionResult> {
        let mut vec_of_text_detection_result = Vec::new();
        if let Some(vector) = self.0.text_detections.take() {
            vector.into_iter().for_each(|text_detection_result| {
                vec_of_text_detection_result.push(text_detection_result);
            });
        }
        vec_of_text_detection_result
    }
    pub fn write_text_detection_results_as_text_and_pdf(&mut self) {
        let job_id = self
            .0
            .job_id
            .take()
            .unwrap_or("No Job ID is presented".into());
        let text_detection_result = self.text_detect_result();
        let headers = vec![
            "Timestamp",
            "Detected text",
            "Text Type",
            "Confidence Level",
        ];
        let (bucket_name, video_key_name) = self.s3_details();
        let mut all_types_results = Vec::new();
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("Get_Text_Detection_Results.txt")
            .expect("Error while Creating File\n");

        text_detection_result.into_iter().for_each(|text_outputs| {
            let timestamp = text_outputs.timestamp();
            let get_text = text_outputs.text_detection;
            if let Some(text_detection) = get_text {
                let mut textdetails = TextDetect::build(text_detection.to_owned());
                let texts = textdetails.detected_text();
                let text_type = textdetails.text_type();
                let confidence = textdetails.confidence();
                let buf = format!("Timestamp: {timestamp}\n");
                file.write_all(buf.as_bytes()).unwrap();
                all_types_results.push(timestamp.to_string());

                if let Some(text) = texts {
                    let buf = format!("Detected Text: {text}\n");
                    file.write_all(buf.as_bytes()).unwrap();
                    all_types_results.push(text);
                }
                if let Some(text_type) = text_type {
                    let buf = format!("Text Type: {text_type}\n");
                    file.write_all(buf.as_bytes()).unwrap();
                    all_types_results.push(text_type)
                }
                if let Some(confidence) = confidence {
                    let buf = format!("Confidence Level: {confidence}\n\n");
                    file.write_all(buf.as_bytes()).unwrap();
                    all_types_results.push(confidence.to_string());
                }
            }
        });
        match std::fs::File::open("Get_Text_Detection_Results.txt"){
            Ok(_) => println!("The text detection results have been successfully written to the current directory with the file name '{}'\n","Get_Text_Detection_Results.txt".green().bold()),
            Err(_) => println!("Error while Writing the data\n")
        }
        create_text_result_pdf(
            &headers,
            all_types_results,
            job_id,
            (bucket_name, video_key_name),
        );
    }
}
impl Deref for GetTextInfo {
    type Target = GetTextDetectionOutput;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FaceDetails(FaceDetail);
impl FaceDetails {
    pub fn build(type_: FaceDetail) -> Self {
        Self(type_)
    }
    /// [`BoundingBox`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.BoundingBox.html)
    pub fn bounding_box(&mut self) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
        let bounding_box = if let Some(bbox) = self.0.bounding_box.take() {
            (bbox.width(), bbox.height(), bbox.left(), bbox.top())
        } else {
            (None, None, None, None)
        };
        bounding_box
    }
    /// [`AgeRange`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.AgeRange.html)
    pub fn age_range(&mut self) -> (Option<i32>, Option<i32>) {
        let age_range = if let Some(age) = self.0.age_range.take() {
            (age.low(), age.high())
        } else {
            (None, None)
        };
        age_range
    }
    /// [`Smile'](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Smile.html)
    pub fn smile(&mut self) -> (Option<bool>, Option<f32>) {
        let smile = if let Some(smile_) = self.0.smile.take() {
            (Some(smile_.value), smile_.confidence)
        } else {
            (None, None)
        };
        smile
    }
    /// [`Eyeglasses`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Eyeglasses.html)
    pub fn eyeglasses(&mut self) -> (Option<bool>, Option<f32>) {
        let eyeglass = if let Some(glass) = self.0.eyeglasses.take() {
            (Some(glass.value), glass.confidence)
        } else {
            (None, None)
        };
        eyeglass
    }
    /// [`Sunglasses`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Sunglasses.html)
    pub fn sunglasses(&mut self) -> (Option<bool>, Option<f32>) {
        let sun_glass = if let Some(sun) = self.0.sunglasses.take() {
            (Some(sun.value), sun.confidence)
        } else {
            (None, None)
        };
        sun_glass
    }
    /// [`Gender`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Gender.html)
    pub fn gender(&mut self) -> (Option<String>, Option<f32>) {
        let gender = if let Some(gender_) = self.0.gender.take() {
            if let (Some(r#type), Some(confidence)) = (gender_.value(), gender_.confidence()) {
                (Some(r#type.as_str().to_string()), Some(confidence))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        gender
    }
    ///[`Beard`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Beard.html)
    pub fn beard(&mut self) -> (Option<bool>, Option<f32>) {
        let beard = if let Some(beard_) = self.0.beard.take() {
            (Some(beard_.value), beard_.confidence)
        } else {
            (None, None)
        };
        beard
    }
    /// [`Mustache`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Mustache.html)
    pub fn mustache(&mut self) -> (Option<bool>, Option<f32>) {
        let mustache = if let Some(mustache_) = self.0.mustache.take() {
            (Some(mustache_.value), mustache_.confidence)
        } else {
            (None, None)
        };
        mustache
    }
}

/// [`FaceDetection`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.FaceDetection.html)
pub struct GetFaceInfo(GetFaceDetectionOutput);
impl GetFaceInfo {
    pub fn job_status(&mut self) -> Option<String> {
        self.0
            .job_status
            .take()
            .map(|job_status| job_status.as_str().to_string())
    }
    pub fn status_message(&mut self) -> Option<String> {
        self.0.status_message.take()
    }
    fn s3_object_details(&mut self) -> (String, String) {
        let mut bucket_name = String::new();
        let mut key_name = String::new();
        self.0.video.take().map(|video| {
            if let Some(s3_object) = video.s3_object {
                if let Some(bucket_name_) = s3_object.bucket() {
                    bucket_name.push_str(bucket_name_);
                }
                if let Some(key_name_) = s3_object.name() {
                    key_name.push_str(key_name_);
                }
            }
        });
        (bucket_name, key_name)
    }
    fn face_detection(&mut self) -> Vec<FaceDetection> {
        let mut vec_of_face_details = Vec::new();
        if let Some(vec_of_face_detail) = self.0.faces.take() {
            vec_of_face_detail.into_iter().for_each(|outputs| {
                vec_of_face_details.push(outputs);
            });
        }
        vec_of_face_details
    }
    pub fn write_face_detection_results_as_text_and_pdf(&mut self) {
        let mut job_id = String::new();
        if let Some(job_id_) = self.0.job_id.take() {
            job_id.push_str(&job_id_);
        }
        let (bucket_name, video_key_name) = self.s3_object_details();
        let headers = vec![
            "Timestamp",
            "Gender and Confidence Level",
            "Age Range in Years",
            "Is the Face Smiling and Confidence Level",
            "Has a Beard and Confidence Level",
            "Has a Mustache and Confidence Level",
            "Has Sunglasses and Confidence Level",
            "Has Eyeglasses and Confidence Level",
            "Bounding Box Details",
        ];
        let mut face_details_vector = Vec::new();
        let face_detail = self.face_detection();
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("Get_Face_Detection_Results.txt")
            .expect("Error while Creating File\n");
        face_detail.into_iter().for_each(|face_details| {
            let timestamp = face_details.timestamp;
            let format_timestamp = format!("Timestamp: {timestamp}\n");
            file.write_all(format_timestamp.as_bytes()).unwrap();
            if let Some(face_detail_type) = face_details.face {
                let mut wrap_face_detection = FaceDetails::build(face_detail_type);
                let gender = wrap_face_detection.gender();
                let mut gender_string = String::new();
                //let format_gender = format!("{:?} ,{:?}", gender.0, gender.1);
               // gender_string.push_str(&format_gender);
                if let (Some(gender_), Some(conf_level)) = (gender.0, gender.1) {
                    gender_string.push_str(&gender_);
                    gender_string.push_str(" and ");
                    let confidence_level = format!("{}", conf_level);
                    gender_string.push_str(&confidence_level);
                } 
                let buf = format!("Gender and Confidence Level: {}\n", gender_string);
                file.write_all(buf.as_bytes()).unwrap();

                let age_range = wrap_face_detection.age_range();
                let mut age_range_string = String::new();
                if let (Some(low),Some(high)) = (age_range.0,age_range.1) {
                    let format_age_range = format!("The lowest age prediction is {low}, and the highest age prediction is {high}");
                    age_range_string.push_str(&format_age_range);
                } 
                let buf = format!("Age Range in Years: {}\n", age_range_string);
                file.write_all(buf.as_bytes()).unwrap();

                let smile = wrap_face_detection.smile();
                let mut smile_string = String::new();
                if let (Some(smiling),Some(conf_level)) = (smile.0,smile.1) {
                    let format_smile = format!("{smiling},{conf_level}");
                    smile_string.push_str(&format_smile);
                }
                let buf = format!(
                    "Is the Face Smiling and Confidence Level: {}\n",
                    smile_string
                );
                file.write_all(buf.as_bytes()).unwrap();

                let beard = wrap_face_detection.beard();
                let mut beard_string = String::new();
                if let (Some(beard),Some(conf_level)) =(beard.0,beard.1)  {
                    let format_beard = format!("{beard},{conf_level}");
                    beard_string.push_str(&format_beard);
                } 
                let buf = format!("Has a Beard and Confidence Level: {}\n", beard_string);
                file.write_all(buf.as_bytes()).unwrap();

                let mustache = wrap_face_detection.mustache();
                let mut mustache_string = String::new();
                if let (Some(mustache),Some(conf_level)) =(mustache.0,mustache.1)  {
                    let format_mustache = format!("{mustache},{conf_level}");
                    mustache_string.push_str(&format_mustache);
                }
                let buf = format!("Has a Mustache and Confidence Level: {}\n", mustache_string);
                file.write_all(buf.as_bytes()).unwrap();

                let sunglasses = wrap_face_detection.sunglasses();
                let mut sunglasses_string = String::new();
                if let (Some(sun),Some(conf_level)) = (sunglasses.0,sunglasses.1) {
                    let format_sunglass = format!("{sun},{conf_level}");
                    sunglasses_string.push_str(&format_sunglass);
                } 
                let buf = format!(
                    "Has Sunglasses and Confidence Level: {}\n",
                    sunglasses_string
                );
                file.write_all(buf.as_bytes()).unwrap();

                let eyeglasses = wrap_face_detection.eyeglasses();
                let mut eyeglasses_string = String::new();
                
                if let (Some(eye),Some(conf_level)) =(eyeglasses.0,eyeglasses.1)  {
                    let format_eyeglasses = format!("{eye},{conf_level}");
                    eyeglasses_string.push_str(&format_eyeglasses);
                } 
                let buf = format!(
                    "Has Eyeglasses and Confidence Level: {}\n",
                    eyeglasses_string
                );
                file.write_all(buf.as_bytes()).unwrap();

                let bounding_box = wrap_face_detection.bounding_box();
                let mut bounding_string = String::new();
                if let (Some(width), Some(height), Some(left), Some(top)) = (
                    bounding_box.0,
                    bounding_box.1,
                    bounding_box.2,
                    bounding_box.3,
                ) {
                    let format_bounding_box =
                        format!("Width: {width}, Height: {height}, Left: {left}, Top: {top}");
                    bounding_string.push_str(&format_bounding_box);
                }
                let buf = format!("Bounding Box Details: {}\n\n\n", bounding_string);
                file.write_all(buf.as_bytes()).unwrap();

                face_details_vector.push(timestamp.to_string());
                face_details_vector.push(gender_string);
                face_details_vector.push(age_range_string);
                face_details_vector.push(smile_string);
                face_details_vector.push(beard_string);
                face_details_vector.push(mustache_string);
                face_details_vector.push(sunglasses_string);
                face_details_vector.push(eyeglasses_string);
                face_details_vector.push(bounding_string);
            }
        });
        match std::fs::File::open("Get_Face_Detection_Results.txt"){
            Ok(_) => println!("The Face detection results have been successfully written to the current directory with the file name '{}'\n","Get_Face_Detection_Results.txt".green().bold()),
            Err(_) => println!("Error while Writing the data\n")
        }
        create_face_result_pdf(
            &headers,
            face_details_vector,
            &job_id,
            (bucket_name, video_key_name),
        );
    }
}
