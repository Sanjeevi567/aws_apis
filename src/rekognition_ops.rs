use std::{fs::OpenOptions, io::Write};

use aws_config::SdkConfig;
use aws_sdk_rekognition::{
    operation::{
        get_face_detection::GetFaceDetectionOutput,
        get_face_liveness_session_results::GetFaceLivenessSessionResultsOutput,
        get_text_detection::GetTextDetectionOutput,
    },
    types::{
        Attribute, CreateFaceLivenessSessionRequestSettings, FaceDetail, FaceDetection, Image,
        LivenessOutputConfig, S3Object, TextDetection, TextDetectionResult, Video,
    },
    Client as RekogClient,
};
use colored::Colorize;
use std::ops::Deref;
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
        let attribute = vec![
            Attribute::AgeRange,
            Attribute::Gender,
            Attribute::Smile,
            Attribute::Beard,
        ];
        let image_builder = Image::builder().s3_object(s3_object_builder).build();
        let detect_face_output = client
            .detect_faces()
            .set_attributes(Some(attribute))
            .image(image_builder)
            .send()
            .await
            .expect("Error while detecting faces");
        let mut vec_of_facedetails = Vec::new();

        if let Some(face_detail) = detect_face_output.face_details {
            face_detail.into_iter().for_each(|outputs| {
                vec_of_facedetails.push(FaceDetails(outputs));
            });
        }
        vec_of_facedetails
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
                    println!("The job ID has been successfully written to the current directory\n")
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
                    println!("The job ID has been successfully written to the current directory\n")
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
            .expect("Error while getting face detection\n");

        GetFaceInfo(get_face_detection_output)
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
    pub fn get_detected_text(&self) -> Option<&str> {
        self.0.detected_text()
    }
    /// [`TextType`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/enum.TextTypes.html)
    pub fn get_text_type(&self) -> Option<&str> {
        self.0.r#type().map(|text_type| text_type.as_str())
    }
    pub fn get_confidence(&self) -> Option<f32> {
        self.0.confidence
    }
    ///[`Geometry`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Geometry.html)
    pub fn get_geometry_region(&self) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
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
    pub fn get_job_status(&self) -> Option<&str> {
        let job_status = if let Some(status) = self.0.job_status.as_ref() {
            Some(status.as_str())
        } else {
            None
        };
        job_status
    }
    pub fn get_status_message(&self) -> Option<&str> {
        self.0.status_message.as_deref()
    }
    pub fn get_text_detect_result(&self) -> Vec<&TextDetectionResult> {
        let mut vec_of_text_detection_result = Vec::new();
        if let Some(vector) = self.0.text_detections.as_ref() {
            vector.iter().for_each(|text_detection_result| {
                vec_of_text_detection_result.push(text_detection_result);
            });
        }
        vec_of_text_detection_result
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
    pub fn get_bounding_box(&self) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
        let bounding_box = if let Some(bbox) = self.0.bounding_box.as_ref() {
            (bbox.width(), bbox.height(), bbox.left(), bbox.top())
        } else {
            (None, None, None, None)
        };
        bounding_box
    }
    /// [`AgeRange`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.AgeRange.html)
    pub fn age_range(&self) -> (Option<i32>, Option<i32>) {
        let age_range = if let Some(age) = self.0.age_range.as_ref() {
            (age.low(), age.high())
        } else {
            (None, None)
        };
        age_range
    }
    /// [`Smile'](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Smile.html)
    pub fn get_smile(&self) -> (Option<bool>, Option<f32>) {
        let smile = if let Some(smile_) = self.0.smile.as_ref() {
            (Some(smile_.value), smile_.confidence)
        } else {
            (None, None)
        };
        smile
    }
    /// [`Eyeglasses`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Eyeglasses.html)
    pub fn get_eyeglasses(&self) -> (Option<bool>, Option<f32>) {
        let eyeglass = if let Some(glass) = self.0.eyeglasses.as_ref() {
            (Some(glass.value), glass.confidence)
        } else {
            (None, None)
        };
        eyeglass
    }
    /// [`Sunglasses`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Sunglasses.html)
    pub fn get_sunglasses(&self) -> (Option<bool>, Option<f32>) {
        let sun_glass = if let Some(sun) = self.0.sunglasses.as_ref() {
            (Some(sun.value), sun.confidence)
        } else {
            (None, None)
        };
        sun_glass
    }
    /// [`Gender`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Gender.html)
    pub fn get_gender(&self) -> (Option<&str>, Option<f32>) {
        let gender = if let Some(gender_) = self.0.gender.as_ref() {
            if let (Some(r#type), Some(confidence)) = (gender_.value(), gender_.confidence()) {
                (Some(r#type.as_str()), Some(confidence))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        gender
    }
    ///[`Beard`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Beard.html)
    pub fn get_beard(&self) -> (Option<bool>, Option<f32>) {
        let beard = if let Some(beard_) = self.0.beard.as_ref() {
            (Some(beard_.value), beard_.confidence)
        } else {
            (None, None)
        };
        beard
    }
    /// [`Mustache`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Mustache.html)
    pub fn get_mustache(&self) -> (Option<bool>, Option<f32>) {
        let mustache = if let Some(mustache_) = self.0.mustache.as_ref() {
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
    pub fn get_job_status(&self) -> Option<&str> {
        self.0.job_status().map(|job_status| job_status.as_str())
    }
    pub fn get_status_message(&self) -> Option<&str> {
        self.0.status_message()
    }
    pub fn get_face_detection(&self) -> Vec<&FaceDetection> {
        let mut vec_of_face_details = Vec::new();
        if let Some(vec_of_face_detail) = self.0.faces.as_ref() {
            vec_of_face_detail.iter().for_each(|outputs| {
                vec_of_face_details.push(outputs);
            });
        }
        vec_of_face_details
    }
}
