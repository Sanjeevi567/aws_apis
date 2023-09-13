use std::{fs::OpenOptions, io::Write};

use aws_config::SdkConfig;
use aws_sdk_memorydb::primitives::DateTimeFormat;
use aws_sdk_polly::{
    primitives::ByteStream,
    types::{Engine, Gender, LanguageCode, OutputFormat, TaskStatus, TextType, VoiceId},
    Client as PollyClient,
};

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

    pub async fn start_synthesise_task(
        &self,
        engine: &str,
        voice_id: &str,
        language_code: &str,
        text_type: &str,
        text_to_synthesize: &str,
        output_format: &str,
        bucket_name: &str,
    ) -> SynthesizeTask {
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
            .send()
            .await
            .expect("Error while start synthesize task");
        let mut synthesizetask = SynthesizeTask::default();

        let synthesize_info = output.synthesis_task;
        if let Some(synthesizeinfo) = synthesize_info {
            let engine = synthesizeinfo.engine;
            let task_status = synthesizeinfo.task_status;
            let output_uri = synthesizeinfo.output_uri;
            let output_format = synthesizeinfo.output_format;
            let text_type = synthesizeinfo.text_type;
            let voice_id = synthesizeinfo.voice_id;
            let language_code = synthesizeinfo.language_code;
            synthesizetask = SynthesizeTask::build_synthesizetask(
                engine,
                task_status,
                output_uri,
                output_format,
                text_type,
                voice_id,
                language_code,
            );
        }

        synthesizetask
    }

    /// Returns a tuple consisting of two vectors: one containing options for [`voice ID`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/types/struct.Voice.html#method.id) and the other containing options for [`language code`](https://docs.aws.amazon.com/polly/latest/dg/API_StartSpeechSynthesisTask.html#polly-StartSpeechSynthesisTask-request-LanguageCode), based on the engine name provided
    pub async fn get_info_given_engine(
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

    pub async fn get_synthesise_tasks(&self) {
        let config = self.get_config();
        let client = PollyClient::new(config);

        let output = client
            .list_speech_synthesis_tasks()
            .max_results(5)
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
                let gender = voice.gender;
                let voice_id = voice.id;
                let language_code = voice.language_code;
                let language_name = voice.language_name;
                let voice_name = voice.name;
                let supported_engines = voice.supported_engines;
                vec_of_voices.push(DescribeVoices::build_describe_voices(
                    gender,
                    voice_id,
                    language_code,
                    language_name,
                    voice_name,
                    supported_engines,
                ));
            });
        }
        vec_of_voices
    }
}

/// A struct for storing information of type [`SynthesisTask`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/types/struct.SynthesisTask.html#) which is returned from the [`start_synthesize_task`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/struct.Client.html#method.start_speech_synthesis_task) REST API
#[derive(Default)]
pub struct SynthesizeTask {
    engine: Option<Engine>,
    task_status: Option<TaskStatus>,
    output_uri: Option<String>,
    output_format: Option<OutputFormat>,
    text_type: Option<TextType>,
    voice_id: Option<VoiceId>,
    language_code: Option<LanguageCode>,
}
impl SynthesizeTask {
    fn build_synthesizetask(
        engine: Option<Engine>,
        task_status: Option<TaskStatus>,
        output_uri: Option<String>,
        output_format: Option<OutputFormat>,
        text_type: Option<TextType>,
        voice_id: Option<VoiceId>,
        language_code: Option<LanguageCode>,
    ) -> Self {
        Self {
            engine,
            task_status,
            output_uri,
            output_format,
            text_type,
            voice_id,
            language_code,
        }
    }
    pub fn get_engine(&self) -> Option<&str> {
        let engine = if let Some(engine) = self.engine.as_ref() {
            Some(engine.as_str())
        } else {
            None
        };
        engine
    }

    pub fn get_task_status(&self) -> Option<&str> {
        let task_status = if let Some(taskstatus) = self.task_status.as_ref() {
            Some(taskstatus.as_str())
        } else {
            None
        };
        task_status
    }

    pub fn get_output_uri(&self) -> Option<&str> {
        let uri = if let Some(uri_) = self.output_uri.as_deref() {
            Some(uri_)
        } else {
            None
        };
        uri
    }

    pub fn get_output_format(&self) -> Option<&str> {
        let format = if let Some(format_) = self.output_format.as_ref() {
            Some(format_.as_str())
        } else {
            None
        };
        format
    }

    pub fn get_text_type(&self) -> Option<&str> {
        let text_type = if let Some(text) = self.text_type.as_ref() {
            Some(text.as_str())
        } else {
            None
        };
        text_type
    }

    pub fn get_voice_id(&self) -> Option<&str> {
        let voice_id = if let Some(id) = self.voice_id.as_ref() {
            Some(id.as_str())
        } else {
            None
        };
        voice_id
    }
    pub fn get_language_code(&self) -> Option<&str> {
        let lang_code = if let Some(lang) = self.language_code.as_ref() {
            Some(lang.as_str())
        } else {
            None
        };
        lang_code
    }
}
/// A struct for storing information of type [`Voice`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/types/struct.Voice.html) which is returned from the [`describe_voices`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/operation/describe_voices/builders/struct.DescribeVoicesFluentBuilder.html) REST API
pub struct DescribeVoices {
    gender: Option<Gender>,
    voice_id: Option<VoiceId>,
    language_code: Option<LanguageCode>,
    language_name: Option<String>,
    voice_name: Option<String>,
    supported_engines: Option<Vec<Engine>>,
}
impl DescribeVoices {
    fn build_describe_voices(
        gender: Option<Gender>,
        voice_id: Option<VoiceId>,
        language_code: Option<LanguageCode>,
        language_name: Option<String>,
        voice_name: Option<String>,
        supported_engines: Option<Vec<Engine>>,
    ) -> Self {
        Self {
            gender,
            voice_id,
            language_code,
            language_name,
            voice_name,
            supported_engines,
        }
    }
    pub fn get_gender(&self) -> Option<&str> {
        let gender = if let Some(gender) = self.gender.as_ref() {
            Some(gender.as_str())
        } else {
            None
        };
        gender
    }
    pub fn get_voiceid(&self) -> Option<&str> {
        let voice_id = if let Some(voiceid) = self.voice_id.as_ref() {
            Some(voiceid.as_str())
        } else {
            None
        };

        voice_id
    }

    pub fn get_language_code(&self) -> Option<&str> {
        let language_code = if let Some(lang_code) = self.language_code.as_ref() {
            Some(lang_code.as_str())
        } else {
            None
        };
        language_code
    }
    pub fn get_language_name(&self) -> Option<&str> {
        let language_name = if let Some(lang_name) = self.language_name.as_ref() {
            Some(lang_name.as_str())
        } else {
            None
        };
        language_name
    }
    pub fn get_voice_name(&self) -> Option<&str> {
        let voice_name = if let Some(voicename) = self.voice_name.as_ref() {
            Some(voicename.as_str())
        } else {
            None
        };
        voice_name
    }
    pub fn get_supported_engines(&self) -> Option<Vec<&str>> {
        let vec_of_engines = if let Some(engines) = self.supported_engines.as_ref() {
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
