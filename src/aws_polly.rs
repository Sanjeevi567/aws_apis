use std::{fs::OpenOptions, io::Write};

use aws_config::SdkConfig;
use aws_sdk_polly::{
    primitives::ByteStream,
    types::{Engine, OutputFormat, VoiceId, TextType},
    Client as PollyClient,
};

use colored::Colorize;

pub struct PollyOps {
    config: SdkConfig,
}
impl PollyOps {
    fn get_config(&self) -> &SdkConfig {
        &self.config
    }
    pub async fn synthesize_speech(
        &self,
        engine: &str,
        output_format: &str,
        text_to_synthesize: &str,
    ) -> SpeechOuputInfo {
        let config = self.get_config();
        let client = PollyClient::new(config);

        let engine_builder = match engine {
            "neural" | "Neural" => Engine::Neural,
            "standard" | "Standard" => Engine::Standard,
            _ => panic!("Invalid engine type: {}", engine),
        };

        let possible_output_formats = "Mp3\nJson\nOggVorbis\npcm\n";
        let ouput_format_builder = match output_format {
            "mp3" | "MP3" => OutputFormat::Mp3,
            "Json" | "json" => OutputFormat::Json,
            "oggvorbis" | "OggVorbis" => OutputFormat::OggVorbis,
            "pcm" | "PCM" => OutputFormat::Pcm,
            _ => panic!(
                "Invalid output format: {}\n Possible Output Formats are: {}\n",
                output_format, possible_output_formats
            ),
        };

        let default_voice_id = VoiceId::Aditi;
        let extension = text_to_synthesize.split('.').last().unwrap();
        let text_type = match extension {
            "ssml" | "Ssml" => TextType::Ssml,
             _ => TextType::Text
        };

        let output = client
            .synthesize_speech()
            .engine(engine_builder)
            .output_format(ouput_format_builder)
            .text(text_to_synthesize)
            .text_type(text_type)
            .voice_id(default_voice_id)
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
}
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
