use aws_config::SdkConfig;
use aws_sdk_transcribe::{
    primitives::DateTimeFormat,
    types::{Media, MediaFormat, SubtitleFormat, Subtitles, TranscriptionJob},
    Client as TranscribeClient,
};
use colored::Colorize;

pub struct TranscribeOps{
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
        let srt_format = SubtitleFormat::Srt;
        let vtt_format = SubtitleFormat::Vtt;
        let vec_of_formats = vec![srt_format, vtt_format];
        let subtitle_builder = Subtitles::builder()
            .set_formats(Some(vec_of_formats))
            .output_start_index(1)
            .build();
        client
            .start_transcription_job()
            .output_bucket_name(bucket_name)
            .output_key("transcribe_outputs/")
            .subtitles(subtitle_builder)
            .media(media_builder)
            .identify_language(true)
            .media_format(media_format_builder)
            .transcription_job_name(job_name)
            .send()
            .await
            .expect("Error while starting transcribing task\n");
        println!(
            "The key prefix is configured as {} for the bucket name: {}\n",
            "'transcribe_outputs/'".green().bold(),
            bucket_name.green().bold()
        );
        println!(
            "{}\n",
            "Both SRT and VTT subtitles are included in the request"
                .yellow()
                .bold()
        );
        println!(
            "The job name {},is used to retrieve the transcribed results in the '{}'\n",
            job_name.green().bold(),
            "Get Transcription Job".yellow().bold()
        );
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
    pub fn job_status(&mut self) -> Option<String> {
        self.0
            .transcription_job_status
            .take()
            .map(|status| status.as_str().to_string())
    }
    fn lang_code(&mut self) -> Option<String> {
        self.0
            .language_code
            .take()
            .map(|code| code.as_str().to_string())
    }
    fn media_format(&mut self) -> Option<String> {
        self.0
            .media_format
            .take()
            .map(|mediat_fmt| mediat_fmt.as_str().to_string())
    }
    fn media(&mut self) -> Option<(Option<String>, Option<String>)> {
        self.0
            .media
            .take()
            .map(|media| (media.media_file_uri, media.redacted_media_file_uri))
    }
    fn transcript_uri(&mut self) -> Option<(Option<String>, Option<String>)> {
        self.0.transcript.take().map(|transcript| {
            (
                transcript.transcript_file_uri,
                transcript.redacted_transcript_file_uri,
            )
        })
    }
    fn start_time(&self) -> Option<String> {
        self.0
            .start_time()
            .map(|date_time| date_time.fmt(DateTimeFormat::HttpDate))
            .map(|option_of_result| option_of_result.ok())
            .flatten()
    }
    fn creation_time(&self) -> Option<String> {
        self.0
            .creation_time()
            .map(|time| time.fmt(DateTimeFormat::HttpDate))
            .map(|get_inner| get_inner.ok())
            .flatten()
    }
    fn completion_time(&self) -> Option<String> {
        self.0
            .completion_time()
            .map(|format_date| format_date.fmt(DateTimeFormat::HttpDate))
            .map(|get_inner| get_inner.ok())
            .flatten()
    }

    pub fn failure_reason(&mut self) -> Option<String> {
        self.0.failure_reason.take()
    }
    //Help message in the bucket url : get the bukcket name then put s3://bucket_name/paste key name
    pub fn print_transcription_info_as_text(&mut self) {
        if let Some(name) = self.0.transcription_job_name.take() {
            println!("Transcription Job Name: {}", name.green().bold());
        }
        if let Some(status) = self.job_status() {
            println!("Transcription Job Status: {}", status.green().bold());
        }

        if let Some(code) = self.lang_code() {
            println!("Language_code: {}", code.green().bold());
        }
        if let Some(media_format_) = self.media_format() {
            println!("Media Format: {}", media_format_.green().bold());
        }
        if let Some(media_) = self.media() {
            println!("Media Information Inlcuded In the Request:\n");
            if let Some(media_uri) = media_.0 {
                println!("Media_file Uri: {}", media_uri.green().bold());
            }

            if let Some(redact) = media_.1 {
                println!("Redacted Media File Uri: {}", redact.green().bold());
            }
        }

        if let Some(uris) = self.transcript_uri() {
            if let Some(trans_uri) = uris.0 {
                println!("Transcript File Uri: {}", trans_uri.green().bold());
            }
            if let Some(redact) = uris.1 {
                println!("Redacted Transcript File Uri: {}", redact.green().bold());
            }
        }
        if let Some(stime) = self.start_time() {
            println!("Start Time: {}", stime.green().bold());
        }
        if let Some(cre_time) = self.creation_time() {
            println!("Creation Time: {}", cre_time.green().bold());
        }
        if let Some(com_time) = self.completion_time() {
            println!("Completion Time: {}", com_time.green().bold());
        }

        if let Some(fail_reason) = self.failure_reason() {
            println!("Failure Reason: {}", fail_reason.green().bold());
        }

        if let Some(sub_output) = self.0.subtitles.take() {
            if let Some(formats) = sub_output.formats {
                formats.into_iter().for_each(|format| {
                    println!("Subtitle Format: {}", format.as_str().green().bold());
                })
            }
            if let Some(sub_uris) = sub_output.subtitle_file_uris {
                sub_uris.into_iter().for_each(|uri| {
                    println!("Subtitle Uri: {}", uri.green().bold());
                });
            };
            if let Some(index) = sub_output.output_start_index {
                println!("Starting Index: {}", index.to_string().green().bold());
            }
            println!("");
            println!("{}\n","The bucket URL cannot be accessed until it is made public or only accessible through the web console".yellow().bold());
            println!("{}\n","To achieve this, please execute the 'Modify Object Visibility' option in the S3 operations menu to make the object public".yellow().bold());
        }
    }
}
