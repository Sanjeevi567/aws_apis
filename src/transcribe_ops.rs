use aws_config::SdkConfig;
use aws_sdk_transcribe::{
    primitives::DateTimeFormat,
    types::{Media, MediaFormat, SubtitleFormat, Subtitles, TranscriptionJob},
    Client as TranscribeClient,
};
use colored::Colorize;

use crate::create_transcription_pdf;
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
        let srt_format = SubtitleFormat::Srt;
        let vtt_format = SubtitleFormat::Vtt;
        let vec_of_formats = vec![srt_format, vtt_format];
        let subtitle_builder = Subtitles::builder()
            .set_formats(Some(vec_of_formats))
            .output_start_index(1)
            .build();
        let output = client
            .start_transcription_job()
            .output_bucket_name(bucket_name)
            .output_key("transcribe_outputs/")
            .subtitles(subtitle_builder)
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
        println!(
            "{}\n",
            "Both SRT and VTT subtitles are included in the request"
                .yellow()
                .bold()
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

    fn failure_reason(&mut self) -> Option<String> {
        self.0.failure_reason.take()
    }
    pub fn write_transcription_info_as_text_and_pdf(&mut self) {
        let headers = vec![
            "Transcription Job Name",
            "Transcription Job Status",
            "language_code",
            "Media Format",
            "Media",
            "Transcript Info",
            "Start Time",
            "Creation Time",
            "Completion Time",
            "Failure Reason",
            "Subtitles",
        ];
        let mut vec_of_values = Vec::new();
        let mut job_name = String::new();
        if let Some(name) = self.0.transcription_job_name.take() {
            job_name.push_str(&name);
        }
        let mut job_status = String::new();
        if let Some(status) = self.job_status() {
            job_status.push_str(&status);
        }
        let mut language_code = String::new();
        if let Some(code) = self.lang_code() {
            language_code.push_str(&code);
        }
        let mut media_format = String::new();
        if let Some(media_format_) = self.media_format() {
            media_format.push_str(&media_format_);
        }
        let mut media = String::new();
        if let Some(media_) = self.media() {
            let mut media_file_uri = String::from("Media File Uri: ");
            if let Some(media_uri) = media_.0 {
                media_file_uri.push_str(&media_uri);
            }
            let mut redacted_media_file_uri = String::from("Redacted Media File Uri: ");
            if let Some(redact) = media_.1 {
                redacted_media_file_uri.push_str(&redact);
            }
            media.push_str(&media_file_uri);
            media.push_str(" , ");
            media.push_str(&redacted_media_file_uri);
        }
        let mut transcripe = String::new();
        if let Some(uris) = self.transcript_uri() {
            let mut transcript_file_uri = String::from("Transcript File Uri: ");
            if let Some(trans_uri) = uris.0 {
                transcript_file_uri.push_str(&trans_uri);
            }
            let mut redacted_transcript_file_uri = String::from("Redacted Transcript File Uri");
            if let Some(redact) = uris.1 {
                redacted_transcript_file_uri.push_str(&redact);
            }
            transcripe.push_str(&transcript_file_uri);
            transcripe.push_str(" , ");
            transcripe.push_str(&redacted_transcript_file_uri);
        }
        let mut start_time = String::new();
        if let Some(stime) = self.start_time() {
            start_time.push_str(&stime);
        }
        let mut creation_time = String::new();
        if let Some(cre_time) = self.creation_time() {
            creation_time.push_str(&cre_time);
        }
        let mut completion_time = String::new();
        if let Some(com_time) = self.completion_time() {
            completion_time.push_str(&com_time);
        }
        let mut failure_reson = String::new();
        if let Some(fail_reason) = self.failure_reason() {
            failure_reson.push_str(&fail_reason);
        }
        let mut subtitles_info = String::new();
        if let Some(sub_output) = self.0.subtitles.take() {
            let mut sub_formats = String::from("Format of Subtitle: ");
            sub_output.formats.into_iter().for_each(|format| {
                format.into_iter().for_each(|format| {
                    sub_formats.push_str(format.as_str());
                });
            });
            let mut subtitle_file_uris = String::from("subtitle_file_uris: ");
            if let Some(sub_uris) = sub_output.subtitle_file_uris {
                sub_uris.into_iter().for_each(|uri| {
                    subtitle_file_uris.push_str(&uri);
                    subtitle_file_uris.push_str(" , ");
                });
            }
            let mut output_start_index = String::from("output_start_index: ");
            if let Some(index) = sub_output.output_start_index {
                output_start_index.push_str(index.to_string().as_str());
            }
            subtitles_info.push_str(&sub_formats);
            subtitles_info.push_str(&subtitle_file_uris);
            subtitles_info.push_str(&output_start_index);
        }
        vec_of_values.push(job_name);
        vec_of_values.push(job_status);
        vec_of_values.push(language_code);
        vec_of_values.push(media_format);
        vec_of_values.push(media);
        vec_of_values.push(transcripe);
        vec_of_values.push(start_time);
        vec_of_values.push(creation_time);
        vec_of_values.push(completion_time);
        vec_of_values.push(failure_reson);
        vec_of_values.push(subtitles_info);

        create_transcription_pdf(&headers, vec_of_values);
    }
}
