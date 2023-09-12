use std::{fs::OpenOptions, io::Write};

use aws_config::SdkConfig;
use aws_sdk_polly::{
    primitives::ByteStream,
    types::{Engine, OutputFormat, VoiceId, TextType, LanguageCode,Gender,},
    Client as PollyClient,
};

use colored::Colorize;

pub struct PollyOps {
    config: SdkConfig,
}
impl PollyOps {

    pub fn build(config:SdkConfig)->Self{
        Self{
            config
        }
    }
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

        let default_voice_id = VoiceId::Bianca;
        let extension = text_to_synthesize.split('.').last().unwrap();
        let text_type = match extension {
            "ssml" | "Ssml" => TextType::Ssml,
             _ => TextType::Text
        };
     
        let language = LanguageCode::EnUs;
        let output = client
            .synthesize_speech()
            .engine(engine_builder)
            .output_format(ouput_format_builder)
            .text(text_to_synthesize)
            .text_type(text_type)
            .voice_id(default_voice_id)
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

    pub async fn describe_voices(&self)->Vec<DescribeVoices>{
    let config = self.get_config();
    let client = PollyClient::new(config);

    let output = client.describe_voices()
               .send().await
               .expect("Error while describing voices\n");
     let mut vec_of_voices = Vec::new();

     if let Some(voices) = output.voices {
        voices.into_iter()
        .for_each(|voice|{
            let gender = voice.gender;
            let voice_id = voice.id;
            let language_code = voice.language_code;
            let language_name = voice.language_name;
            let voice_name = voice.name;
            let supported_engines = voice.supported_engines;
           vec_of_voices.push(DescribeVoices::build_describe_voices(gender, voice_id, language_code, 
            language_name, voice_name, supported_engines));
        });
         
     }  
      vec_of_voices     
    
    
    }
}

/// A struct for storing information of type [`Voice`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/types/struct.Voice.html) which is returned from the [`describe_voices`](https://docs.rs/aws-sdk-polly/latest/aws_sdk_polly/operation/describe_voices/builders/struct.DescribeVoicesFluentBuilder.html) REST API
pub struct DescribeVoices{
    gender : Option<Gender>,
    voice_id : Option<VoiceId>,
    language_code : Option<LanguageCode>,
    language_name : Option<String>,
    voice_name : Option<String>,
    supported_engines : Option<Vec<Engine>>,
}
impl DescribeVoices{
    fn build_describe_voices(gender: Option<Gender>,voice_id:Option<VoiceId>,
    language_code : Option<LanguageCode>,language_name : Option<String>,
    voice_name : Option<String>,supported_engines : Option<Vec<Engine>>
    ) -> Self{
        Self { gender,
             voice_id,
            language_code,
            language_name, 
            voice_name,
            supported_engines
         }
    }
    pub fn get_gender(&self)->Option<&str>{
        let gender = if let Some(gender) = self.gender.as_ref(){
                 Some(gender.as_str())
        }else{
                  None
        };
        gender
    }
    pub fn get_voiceid(&self)->Option<&str>{
        let voice_id = if let Some(voiceid) = self.voice_id.as_ref() {
            Some(voiceid.as_str())
        }else{
            None
        };

        voice_id
    }

    pub fn get_language_code(&self)->Option<&str>{
        let language_code = if let Some(lang_code) = self.language_code.as_ref() {
            Some(lang_code.as_str())
        }else{
            None
        };
        language_code
    }
    pub fn get_language_name(&self)->Option<&str>{
        let language_name = if let Some(lang_name) = self.language_name.as_ref()  {
            Some(lang_name.as_str())
        }else{
            None
        };
        language_name
    }
    pub fn get_voice_name(&self)->Option<&str>{
        let voice_name = if let Some(voicename) = self.voice_name.as_ref() {
            Some(voicename.as_str())
        }else{
            None
        };
        voice_name
    }
    pub fn get_supported_engines(&self)->Option<Vec<&str>>{
        let vec_of_engines = if let Some(engines) = self.supported_engines.as_ref() {
           let mut vec_of_engine = Vec::new();  
           engines.iter()
           .for_each(|engines|{
            vec_of_engine.push(engines.as_str());
           });
           Some(vec_of_engine)
        }else{
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
