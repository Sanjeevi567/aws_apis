mod credentials;
pub use credentials::{load_credential_from_env, CredentInitialize};

mod memorydb_ops;
pub use memorydb_ops::{MemDbClusterInfo, MemDbOps};

mod rds_ops;
pub use rds_ops::{DbClusterInfo, DbInstanceInfo, RdsOps};

mod s3_ops;
pub use s3_ops::S3Ops;

mod sesv2_ops;
pub use sesv2_ops::{
    SesOps, SimpleMail,
    SimpleOrTemplate::{Simple_, Template_},
    TemplateMail,
};

mod aws_polly;
pub use aws_polly::PollyOps;

mod pinpoint_ops;
pub use pinpoint_ops::PinPointOps;

mod sns_ops;
pub use sns_ops::SnsOps;

mod rekognition_ops;
pub use rekognition_ops::{FaceDetails, RekognitionOps, TextDetect};

mod transcribe_ops;
pub use transcribe_ops::TranscribeOps;

mod route53;
pub use route53::Route53Ops;

mod dynamodb;
pub use dynamodb::DynamoDbOps;

mod pdf_writer;
pub use pdf_writer::{
    create_celebrity_pdf, create_celebrity_single_pdf, create_detect_face_image_pdf, create_polly_voice_info_pdf,
    create_email_identities_pdf, create_email_pdf, create_face_result_pdf, create_text_result_pdf,
};
