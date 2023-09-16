use std::{fs::OpenOptions, io::Write, time::SystemTime};

use aws_config::SdkConfig;
use aws_sdk_memorydb::primitives::DateTimeFormat;
use aws_sdk_polly::{
    primitives::ByteStream,
    types::{
        Engine, LanguageCode, OutputFormat, SynthesisTask, TaskStatus, TextType, Voice, VoiceId,
    },
    Client as PollyClient,
};
use std::ops::Deref;

use colored::Colorize;

pub struct PollyOps {
    config: SdkConfig,
}
impl PollyOps {
    pub fn build(config: SdkConfig) -> Self {
        Self { config }
    }
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }
    pub async fn synthesize_speech(
        &self,
        engine: &str,
        output_format: &str,
        text_to_synthesize: &str,
        voice_id: &str,
        text_type: &str,
    ) -> SpeechOuputInfo {
        let config = self.get_config();
        let client = PollyClient::new(config);

        let engine_builder = Engine::from(engine);

        let ouput_format_builder = OutputFormat::from(output_format);

        let voice_id_builder = VoiceId::from(voice_id);

        let text_type_builder = TextType::from(text_type);

        let language = LanguageCode::EnUs;
        let output = client
            .synthesize_speech()
            .engine(engine_builder)
            .output_format(ouput_format_builder)
            .text(text_to_synthesize)
            .text_type(text_type_builder)
            .voice_id(voice_id_builder)
            .language_code(language)
            .send()
            .await
            .expect("Error while synthesizing speech\n");

        let speech_bytes = output.audio_stream;
        let character_synthesized = output.request_characters;
        let content_type = output.content_type;

        SpeechOuputInfo::build_speech_output_info(
            Some(speech_bytes),
            character_synthesized,
            content_type,
        )
    }

    pub async fn start_speech_synthesise_task(
        &self,
        engine: &str,
        voice_id: &str,
        language_code: &str,
        text_type: &str,
        text_to_synthesize: &str,
        output_format: &str,
        bucket_name: &str,
    ) {
        let config = self.get_config();
        let client = PollyClient::new(config);

        let engine_builder = Engine::from(engine);

        let output_format_builder = OutputFormat::from(output_format);

        let voice_id_builder = VoiceId::from(voice_id);

        let text_type_builder = TextType::from(text_type);

        let language_code_builder = LanguageCode::from(language_code);

        let output = client
            .start_speech_synthesis_task()
            .engine(engine_builder)
            .voice_id(voice_id_builder)
            .language_code(language_code_builder)
            .output_format(output_format_builder)
            .text(text_to_synthesize)
            .text_type(text_type_builder)
            .output_s3_bucket_name(bucket_name)
            .output_s3_key_prefix("speech_synthesis_task_outputs/")
            .send()
            .await
            .expect("Error while start synthesize task\n");
        let key_prefix = "'speech_synthesis_task_outputs/'".green().bold();
        println!(
            "The key prefix in the provided bucket is set to: {key_prefix}\n.
        "
        );

        let synthesize_info = output.synthesis_task;
        if let Some(synthesizeinfo) = synthesize_info {
            let task_id = synthesizeinfo.task_id;
            if let Some(task_id) = task_id {
                println!(
                    "The Task ID for initiating speech synthesis tasks is as follows: {}\n",
                    task_id.green().bold()
                );

                let mut file = OpenOptions::new()
                    .create(true)
                    .read(true)
                    .write(true)
                    .open("task_id.txt")
                    .expect("Error while creating file\n");
                let buf = format!("The speech synthesis task with ID {task_id} has been initiated for the bucket named {bucket_name}");
                match file.write_all(buf.as_bytes()) {
                    Ok(_) => println!("{}\n","The Task ID for the speech synthesis task has been successfully written to the current directory".green().bold()),
                    Err(_) => println!("Error while writing data\n")
                }
            }
        }
    }
    pub async fn get_speech_synthesis_result(&self, task_id: &str) -> Option<SynthesizeTask> {
        let config = self.get_config();
        let client = PollyClient::new(config);
        let output = client
            .get_speech_synthesis_task()
            .task_id(task_id)
            .send()
            .await
            .expect("Error while getting speech synthesis outputs\n");

        let mut synthesis_task: Option<SynthesizeTask> = None;
        if let Some(task) = output.synthesis_task {
            let type_ = SynthesizeTask::wrap(task);
            synthesis_task = Some(type_);
        }
        synthesis_task
    }

    /// Returns a tuple consisting of two vectors: one containing options for [`voice ID`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/types/struct.Voice.html#method.id) and the other containing options for [`language code`](https://docs.aws.amazon.com/polly/latest/dg/API_StartSpeechSynthesisTask.html#polly-StartSpeechSynthesisTask-request-LanguageCode), based on the engine name provided
    pub async fn get_voice_info_given_engine(
        &self,
        engine_name: &str,
    ) -> (Vec<Option<VoiceId>>, Vec<Option<LanguageCode>>) {
        let config = self.get_config();
        let client = PollyClient::new(config);

        let output = client
            .describe_voices()
            .set_engine(Some(engine_name.into()))
            .send()
            .await
            .expect("Error while get information about voices");
        let voices = output.voices;
        let mut supported_voice_id = Vec::new();
        let mut supported_langauge_name = Vec::new();

        if let Some(voices_) = voices {
            voices_.into_iter().for_each(|voice| {
                supported_voice_id.push(voice.id);
                supported_langauge_name.push(voice.language_code);
            });
        }
        (supported_voice_id, supported_langauge_name)
    }
    /// List the synthesis tasks. The status is hardcoded as 'Completed,' meaning it only returns tasks that are in the 'Completed' state. However, for other states, you need to obtain input from the caller and construct the [`TaskStatus`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/types/enum.TaskStatus.html) using the [`from`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/types/enum.TaskStatus.html#impl-From%3C%26str%3E-for-TaskStatus) method
    pub async fn list_synthesise_speech(&self) {
        let config = self.get_config();
        let client = PollyClient::new(config);

        let status_builder = TaskStatus::Completed;

        let output = client
            .list_speech_synthesis_tasks()
            .status(status_builder)
            .send()
            .await
            .expect("Error while listing synthesise tasks");
        let info = output.synthesis_tasks;
        let format = DateTimeFormat::HttpDate;
        if let Some(vec_of_tasks) = info {
            println!("Synthesize Task Details\n\n");
            vec_of_tasks.into_iter().for_each(|task| {
                let creation_time = task.creation_time;
                let task_id = task.task_id;
                let status_reason = task.task_status_reason;
                let task_status = task.task_status;
                let output_uri = task.output_uri;
                if let (Some(time), Some(id), Some(status), Some(uri), Some(reason)) = (
                    creation_time,
                    task_id,
                    task_status,
                    output_uri,
                    status_reason,
                ) {
                    let time_format = time.fmt(format).expect("Error while getting time");
                    let colored_time = time_format.green().bold();
                    let colored_id = id.green().bold();
                    let colored_status = status.as_str().green().bold();
                    let colored_reason = reason.green().bold();
                    let colored_url = uri.green().bold();
                    println!("Creation Time: {colored_time}\n");
                    println!("Task ID: {colored_id}\n");
                    println!("Task Status: {colored_status}\n");
                    println!("Task Status Reason: {colored_reason}\n");
                    println!("Output URL: {colored_url}\n");
                }
            });
        }
    }
    pub async fn describe_voices(&self) -> Vec<DescribeVoices> {
        let config = self.get_config();
        let client = PollyClient::new(config);

        let output = client
            .describe_voices()
            .send()
            .await
            .expect("Error while describing voices\n");
        let mut vec_of_voices = Vec::new();

        if let Some(voices) = output.voices {
            voices.into_iter().for_each(|voice| {
                vec_of_voices.push(DescribeVoices::wrap(voice));
            });
        }
        vec_of_voices
    }
}

/// A wrapper struct for storing information of type [`SynthesisTask`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/types/struct.SynthesisTask.html#) which is returned from the [`start_synthesize_task`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/struct.Client.html#method.start_speech_synthesis_task) REST API
pub struct SynthesizeTask(SynthesisTask);
impl Deref for SynthesizeTask {
    type Target = SynthesisTask;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl SynthesizeTask {
    pub fn wrap(type_: SynthesisTask) -> Self {
        Self(type_)
    }
    pub fn get_task_id(&self) -> Option<&str> {
        self.0.task_id()
    }
    pub fn get_engine(&self) -> Option<&str> {
        let engine = if let Some(engine) = self.0.engine.as_ref() {
            Some(engine.as_str())
        } else {
            None
        };
        engine
    }

    pub fn get_task_status(&self) -> Option<&str> {
        let task_status = if let Some(taskstatus) = self.0.task_status.as_ref() {
            Some(taskstatus.as_str())
        } else {
            None
        };
        task_status
    }

    pub fn get_output_uri(&self) -> Option<&str> {
        let uri = if let Some(uri_) = self.0.output_uri.as_deref() {
            Some(uri_)
        } else {
            None
        };
        uri
    }

    pub fn get_output_format(&self) -> Option<&str> {
        let format = if let Some(format_) = self.0.output_format.as_ref() {
            Some(format_.as_str())
        } else {
            None
        };
        format
    }

    pub fn get_text_type(&self) -> Option<&str> {
        let text_type = if let Some(text) = self.0.text_type.as_ref() {
            Some(text.as_str())
        } else {
            None
        };
        text_type
    }

    pub fn get_voice_id(&self) -> Option<&str> {
        let voice_id = if let Some(id) = self.0.voice_id.as_ref() {
            Some(id.as_str())
        } else {
            None
        };
        voice_id
    }
    pub fn get_language_code(&self) -> Option<&str> {
        let lang_code = if let Some(lang) = self.0.language_code.as_ref() {
            Some(lang.as_str())
        } else {
            None
        };
        lang_code
    }
}
/// A wrapper struct for storing information of type [`Voice`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/types/struct.Voice.html) which is returned from the [`describe_voices`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/operation/describe_voices/builders/struct.DescribeVoicesFluentBuilder.html) REST API
pub struct DescribeVoices(Voice);

impl Deref for DescribeVoices {
    type Target = Voice;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DescribeVoices {
    pub fn wrap(type_: Voice) -> Self {
        Self(type_)
    }
    pub fn get_gender(&self) -> Option<&str> {
        self.0.gender().map(|gender| gender.as_str())
    }
    pub fn get_voiceid(&self) -> Option<&str> {
        self.0.id().map(|voice_id| voice_id.as_str())
    }

    pub fn get_language_code(&self) -> Option<&str> {
        self.0.language_code().map(|code| code.as_str())
    }
    pub fn get_language_name(&self) -> Option<&str> {
        self.0.language_name()
    }
    pub fn get_voice_name(&self) -> Option<&str> {
        self.0.name()
    }
    pub fn get_supported_engines(&self) -> Option<Vec<&str>> {
        let vec_of_engines = if let Some(engines) = self.0.supported_engines() {
            let mut vec_of_engine = Vec::new();
            engines.iter().for_each(|engines| {
                vec_of_engine.push(engines.as_str());
            });
            Some(vec_of_engine)
        } else {
            None
        };
        vec_of_engines
    }
}
/// A struct for storing information of type [`SynthesizeSpeechOutput`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/operation/synthesize_speech/struct.SynthesizeSpeechOutput.html) which is returned from the [`synthesize_speech`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/struct.Client.html#method.synthesize_speech) REST API.
pub struct SpeechOuputInfo {
    speech_bytes: Option<ByteStream>,
    character_synthesized: i32,
    content_type: Option<String>,
}
impl SpeechOuputInfo {
    fn build_speech_output_info(
        speech_bytes: Option<ByteStream>,
        character_synthesized: i32,
        content_type: Option<String>,
    ) -> Self {
        Self {
            speech_bytes,
            character_synthesized,
            content_type,
        }
    }

    pub async fn generate_audio(&mut self) {
        let bytestream = self.speech_bytes.take().unwrap();
        let extenstion = if let Some(content_type) = self.content_type.as_deref() {
            Some(content_type.split('/').skip(1).collect::<String>())
        } else {
            None
        };
        if let Some(extension_) = extenstion {
            let path_name = format!("synthesized_audio.{}", extension_);
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(&path_name)
                .expect("Error while creating file in the current directory");
            let bytes = bytestream
                .collect()
                .await
                .expect("Error while converting to bytes")
                .into_bytes();
            match file.write_all(&bytes) {
                Ok(_) => {
                    let colored_msg =format!("An audio file with the extension {extension_} has been successfully written to the current directory\n")
                        .green().bold();
                    println!("{colored_msg}");
                }
                Err(_) => println!("Error while writing data.."),
            }
        }
    }
    pub fn get_synthesized_count(&self) -> i32 {
        self.character_synthesized
    }
    pub fn get_content_type(&self) -> Option<String> {
        self.content_type.clone()
    }
}
