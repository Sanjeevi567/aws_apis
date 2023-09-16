use aws_config::SdkConfig;
use aws_sdk_transcribe::{
    primitives::{DateTime, DateTimeFormat},
    types::{Media, MediaFormat, Transcript, TranscriptionJob},
    Client as TranscribeClient,
};
use colored::Colorize;
pub struct TranscribeOps {
    config: SdkConfig,
}
impl TranscribeOps {
    pub fn build(config: SdkConfig) -> Self {
        Self { config }
    }
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }
    pub async fn start_transcribe_task(
        &self,
        bucket_name: &str,
        s3_media_uri: &str,
        media_format: &str,
        job_name: &str,
    ) {
        let config = self.get_config();
        let client = TranscribeClient::new(config);
        let media_builder = Media::builder().media_file_uri(s3_media_uri).build();
        let media_format_builder = MediaFormat::from(media_format);
        let output = client
            .start_transcription_job()
            .output_bucket_name(bucket_name)
            .output_key("transcribe_outputs/")
            .media(media_builder)
            .media_format(media_format_builder)
            .transcription_job_name(job_name)
            .send()
            .await
            .expect("Error while starting transcribing task\n");
        if let Some(job) = output.transcription_job {
            let status = job.transcription_job_status();
            if let Some(status) = status {
                print!(
                    "The job status is as follows: {}",
                    status.as_str().green().bold()
                );
            }
        }
        println!(
            "The key prefix is configured as {} for the bucket name: {}\n",
            "'transcribe_outputs/'".green().bold(),
            bucket_name.green().bold()
        );
        print!("The job name {},is used to retrieve the transcribed results in the GetTranscriptionJob API\n",job_name.green().bold());
    }
    pub async fn get_transcribe_results(&self, job_name: &str) -> Option<TranscriptionOutput> {
        let config = self.get_config();
        let client = TranscribeClient::new(config);

        let output = client
            .get_transcription_job()
            .transcription_job_name(job_name)
            .send()
            .await
            .expect("Error while getting transcribe results\n");

        let mut transcription_job_output: Option<TranscriptionOutput> = None;

        if let Some(results) = output.transcription_job {
            let wrap_type = TranscriptionOutput::wrap(results);
            transcription_job_output = Some(wrap_type);
        }
        transcription_job_output
    }
}
pub struct TranscriptionOutput(TranscriptionJob);
impl TranscriptionOutput {
    pub fn wrap(type_: TranscriptionJob) -> Self {
        Self(type_)
    }
    pub fn job_status(&self) -> Option<&str> {
        self.0.transcription_job_name()
    }
    pub fn start_time(&self) -> Option<String> {
        self.0
            .start_time()
            .map(|date_time| date_time.fmt(DateTimeFormat::HttpDate))
            .map(|option_of_result| option_of_result.ok())
            .flatten()
    }
    pub fn creation_time(&self) -> Option<String> {
        self.0
            .creation_time()
            .map(|time| time.fmt(DateTimeFormat::HttpDate))
            .map(|get_inner| get_inner.ok())
            .flatten()
    }
    pub fn completion_time(&self) -> Option<String> {
        self.0
            .completion_time()
            .map(|format_date| format_date.fmt(DateTimeFormat::HttpDate))
            .map(|get_inner| get_inner.ok())
            .flatten()
    }
    pub fn transcript(&self) -> Option<&str> {
        self.0
            .transcript()
            .map(|transcript| transcript.transcript_file_uri())
            .flatten()
    }
}
