use std::{fs::OpenOptions, io::Write};

use aws_config::SdkConfig;
use aws_sdk_rekognition::{
    types::{
        AgeRange, Attribute, Beard, BoundingBox, Eyeglasses, FaceDetection, Gender, Geometry,
        Image, Mustache, NotificationChannel, S3Object, Smile, Sunglasses, TextDetectionResult,
        TextTypes, Video, VideoJobStatus,
    },
    Client as RekogClient,
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
        let attribute = vec![
            Attribute::AgeRange,
            Attribute::Gender,
            Attribute::Smile,
            Attribute::Beard,
        ];
        let image_builder = Image::builder().s3_object(s3_object_builder).build();
        let output = client
            .detect_faces()
            .set_attributes(Some(attribute))
            .image(image_builder)
            .send()
            .await
            .expect("Error while detecting faces");
        let mut facedetail = Vec::new();
        let face_details = output.face_details;

        if let Some(vec_of_details) = face_details {
            vec_of_details.into_iter().for_each(|detail| {
                let bounding_box = detail.bounding_box;
                let age_range = detail.age_range;
                let smile = detail.smile;
                let eyeglasses = detail.eyeglasses;
                let sunglasses = detail.sunglasses;
                let gender = detail.gender;
                let beard = detail.beard;
                let mustache = detail.mustache;

                facedetail.push(FaceDetails::build_face_details(
                    bounding_box,
                    age_range,
                    smile,
                    eyeglasses,
                    sunglasses,
                    gender,
                    beard,
                    mustache,
                ));
            });
        }
        facedetail
    }

    pub async fn detect_text(&self, bucket_name: &str, key_name: &str) -> Vec<TextDetection> {
        let config = self.get_config();
        let client = RekogClient::new(config);

        let s3_object_builder = S3Object::builder()
            .bucket(bucket_name)
            .name(key_name)
            .build();

        let image_builder = Image::builder().s3_object(s3_object_builder).build();

        let output = client
            .detect_text()
            .image(image_builder)
            .send()
            .await
            .expect("Error while detetecting text\n");
        let text_detection = output.text_detections;
        let mut textdetection = Vec::new();
        if let Some(vec_of_texts) = text_detection {
            vec_of_texts.into_iter().for_each(|texts| {
                let detected_text = texts.detected_text;
                let r#type = texts.r#type;
                let confidence = texts.confidence;
                let geometry = texts.geometry;
                textdetection.push(TextDetection::build_text_detection(
                    detected_text,
                    r#type,
                    confidence,
                    geometry,
                ));
            });
        }
        textdetection
    }
    pub async fn start_text_detection(
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
    pub async fn get_start_text_detection(&self, text_job_id: &str) -> GetTextOuput {
        let config = self.get_config();
        let client = RekogClient::new(config);
        let output = client
            .get_text_detection()
            .job_id(text_job_id)
            .send()
            .await
            .expect("Error while getting text detection\n");
        let job_status = output.job_status;
        let status_message = output.status_message;
        let text_detection = output.text_detections;

        GetTextOuput::build_get_text_output(job_status, status_message, text_detection)
    }
    pub async fn start_face_detection(
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

    pub async fn get_face_detection(&self, face_job_id: &str) -> GetFaceOutput {
        let config = self.get_config();
        let client = RekogClient::new(config);

        let output = client
            .get_face_detection()
            .job_id(face_job_id)
            .send()
            .await
            .expect("Error while getting face detection\n");

        let job_status = output.job_status;
        let status_message = output.status_message;
        let faces = output.faces;
        GetFaceOutput::build_get_face_output(job_status, status_message, faces)
    }
}

pub struct GetTextOuput {
    job_status: Option<VideoJobStatus>,
    status_message: Option<String>,
    text_detection: Option<Vec<TextDetectionResult>>,
}
impl GetTextOuput {
    fn build_get_text_output(
        job_status: Option<VideoJobStatus>,
        status_message: Option<String>,
        text_detection: Option<Vec<TextDetectionResult>>,
    ) -> Self {
        Self {
            job_status,
            status_message,
            text_detection,
        }
    }
    pub fn get_job_status(&self) -> Option<&str> {
        let job_status = if let Some(status) = self.job_status.as_ref() {
            Some(status.as_str())
        } else {
            None
        };
        job_status
    }
    pub fn get_status_message(&self) -> Option<&str> {
        let status_message = if let Some(status) = self.status_message.as_deref() {
            Some(status)
        } else {
            None
        };
        status_message
    }
    pub fn get_text_detect_result(&self) -> Vec<TextDetectionResultType> {
        let mut text_detection_result_ = Vec::new();
        if let Some(text_result) = self.text_detection.clone() {
            text_result.into_iter().for_each(|outputs| {
                let timestamp = outputs.timestamp;
                let text_detection = outputs.text_detection;
                if let Some(text_detection_) = text_detection {
                    let detected_text = text_detection_.detected_text;
                    let r#type = text_detection_.r#type;
                    let confidence = text_detection_.confidence;
                    let geometry = text_detection_.geometry;
                    let text_detectionn = TextDetection::build_text_detection(
                        detected_text,
                        r#type,
                        confidence,
                        geometry,
                    );
                    text_detection_result_.push(
                        TextDetectionResultType::build_text_detection_result(
                            timestamp,
                            text_detectionn,
                        ),
                    );
                }
            });
        }
        text_detection_result_
    }
}
/// [`TextDetectionResult`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.TextDetectionResult.html)
pub struct TextDetectionResultType {
    timestamp: i64,
    text_detection: TextDetection,
}

impl TextDetectionResultType {
    fn build_text_detection_result(timestamp: i64, text_detection: TextDetection) -> Self {
        Self {
            timestamp,
            text_detection,
        }
    }
    pub fn get_text_detection_type(&self) -> &TextDetection {
        &self.text_detection
    }
    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}

/// [`TextDetection`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.TextDetection.html)
pub struct TextDetection {
    detected_text: Option<String>,
    r#type: Option<TextTypes>,
    confidence: Option<f32>,
    geometry: Option<Geometry>,
}
impl TextDetection {
    pub fn build_text_detection(
        detected_text: Option<String>,
        r#type: Option<TextTypes>,
        confidence: Option<f32>,
        geometry: Option<Geometry>,
    ) -> Self {
        Self {
            detected_text,
            r#type,
            confidence,
            geometry,
        }
    }
    pub fn get_detected_text(&self) -> Option<&str> {
        let text = if let Some(text_) = self.detected_text.as_deref() {
            Some(text_)
        } else {
            None
        };
        text
    }
    /// [`TextType`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/enum.TextTypes.html)
    pub fn get_text_type(&self) -> Option<&str> {
        let text_type = if let Some(text_type_) = self.r#type.as_ref() {
            Some(text_type_.as_str())
        } else {
            None
        };
        text_type
    }
    pub fn get_confidence(&self) -> Option<f32> {
        let confidence = if let Some(confidence_) = self.confidence {
            Some(confidence_)
        } else {
            None
        };
        confidence
    }
    ///[`Geometry`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Geometry.html)
    pub fn get_geometry_region(&self) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
        let region = if let Some(region_) = self.geometry.as_ref() {
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

pub struct GetFaceOutput {
    job_status: Option<VideoJobStatus>,
    status_message: Option<String>,
    faces: Option<Vec<FaceDetection>>,
}
impl GetFaceOutput {
    fn build_get_face_output(
        job_status: Option<VideoJobStatus>,
        status_message: Option<String>,
        faces: Option<Vec<FaceDetection>>,
    ) -> Self {
        Self {
            job_status,
            status_message,
            faces,
        }
    }
    pub fn get_job_status(&self) -> Option<&str> {
        let job_status = if let Some(status) = self.job_status.as_ref() {
            Some(status.as_str())
        } else {
            None
        };
        job_status
    }
    pub fn get_status_message(&self) -> Option<&str> {
        let status_message = if let Some(status) = self.status_message.as_deref() {
            Some(status)
        } else {
            None
        };
        status_message
    }
    pub fn get_face_detection_type(&self) -> Vec<FaceDetectionType> {
        let mut face_detection_type = Vec::new();
        if let Some(vec_of_details) = self.faces.clone() {
            vec_of_details.into_iter().for_each(|outputs| {
                let timestamp = outputs.timestamp;
                if let Some(facedetail) = outputs.face {
                    let bounding_box = facedetail.bounding_box;
                    let age_range = facedetail.age_range;
                    let smile = facedetail.smile;
                    let eyeglasses = facedetail.eyeglasses;
                    let sunglasses = facedetail.sunglasses;
                    let gender = facedetail.gender;
                    let beard = facedetail.beard;
                    let mustache = facedetail.mustache;
                    let face_details_type = FaceDetails::build_face_details(
                        bounding_box,
                        age_range,
                        smile,
                        eyeglasses,
                        sunglasses,
                        gender,
                        beard,
                        mustache,
                    );
                    face_detection_type.push(FaceDetectionType::build_face_detection_type(
                        timestamp,
                        face_details_type,
                    ));
                }
            });
        }
        face_detection_type
    }
}
/// [`FaceDetection`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.FaceDetection.html)
pub struct FaceDetectionType {
    timestamp: i64,
    face_detection: FaceDetails,
}
impl FaceDetectionType {
    fn build_face_detection_type(timestamp: i64, face_detection: FaceDetails) -> Self {
        Self {
            timestamp,
            face_detection,
        }
    }
    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
    pub fn get_face_detail_type(&self) -> &FaceDetails {
        &self.face_detection
    }
}

pub struct FaceDetails {
    bounding_box: Option<BoundingBox>,
    age_range: Option<AgeRange>,
    smile: Option<Smile>,
    eyeglasses: Option<Eyeglasses>,
    sunglasses: Option<Sunglasses>,
    gender: Option<Gender>,
    beard: Option<Beard>,
    mustache: Option<Mustache>,
}
impl FaceDetails {
    fn build_face_details(
        bounding_box: Option<BoundingBox>,
        age_range: Option<AgeRange>,
        smile: Option<Smile>,
        eyeglasses: Option<Eyeglasses>,
        sunglasses: Option<Sunglasses>,
        gender: Option<Gender>,
        beard: Option<Beard>,
        mustache: Option<Mustache>,
    ) -> Self {
        Self {
            bounding_box,
            age_range,
            smile,
            eyeglasses,
            sunglasses,
            gender,
            beard,
            mustache,
        }
    }
    /// [`BoundingBox`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.BoundingBox.html)
    pub fn get_bounding_box(&self) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
        let bounding_box = if let Some(bbox) = self.bounding_box.as_ref() {
            (bbox.width(), bbox.height(), bbox.left(), bbox.top())
        } else {
            (None, None, None, None)
        };
        bounding_box
    }
    /// [`AgeRange`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.AgeRange.html)
    pub fn age_range(&self) -> (Option<i32>, Option<i32>) {
        let age_range = if let Some(age) = self.age_range.as_ref() {
            (age.low(), age.high())
        } else {
            (None, None)
        };
        age_range
    }
    /// [`Smile'](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Smile.html)
    pub fn get_smile(&self) -> (Option<bool>, Option<f32>) {
        let smile = if let Some(smile_) = self.smile.as_ref() {
            (Some(smile_.value), smile_.confidence)
        } else {
            (None, None)
        };
        smile
    }
    /// [`Eyeglasses`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Eyeglasses.html)
    pub fn get_eyeglasses(&self) -> (Option<bool>, Option<f32>) {
        let eyeglass = if let Some(glass) = self.eyeglasses.as_ref() {
            (Some(glass.value), glass.confidence)
        } else {
            (None, None)
        };
        eyeglass
    }
    /// [`Sunglasses`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Sunglasses.html)
    pub fn get_sunglasses(&self) -> (Option<bool>, Option<f32>) {
        let sun_glass = if let Some(sun) = self.sunglasses.as_ref() {
            (Some(sun.value), sun.confidence)
        } else {
            (None, None)
        };
        sun_glass
    }
    /// [`Gender`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Gender.html)
    pub fn get_gender(&self) -> (Option<&str>, Option<f32>) {
        let gender = if let Some(gender_) = self.gender.as_ref() {
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
        let beard = if let Some(beard_) = self.beard.as_ref() {
            (Some(beard_.value), beard_.confidence)
        } else {
            (None, None)
        };
        beard
    }
    /// [`Mustache`](https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/types/struct.Mustache.html)
    pub fn get_mustache(&self) -> (Option<bool>, Option<f32>) {
        let mustache = if let Some(mustache_) = self.mustache.as_ref() {
            (Some(mustache_.value), mustache_.confidence)
        } else {
            (None, None)
        };
        mustache
    }
}
