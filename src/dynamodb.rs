use aws_config::SdkConfig;
use aws_sdk_dynamodb::{
    types::{
        AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
    },
    Client as DynClient,
};
use colored::Colorize;
use serde_json::Value;
use std::{fs::File, io::BufReader};
pub struct DynamoDbOps {
    config: SdkConfig,
}
impl DynamoDbOps {
    pub fn build(config:SdkConfig) -> Self {
        Self { config }
    }
    fn get_config(&self)->&SdkConfig{
        &self.config
    }
    pub async fn create_table(
        &self,
        table_name: &str,
        attribute_definition_json_path: &str,
        key_schema_json_path: &str,
    ) {
        let client = DynClient::new(self.get_config());

        let attribute_definitions = parse_attribute_defintion_json(attribute_definition_json_path);
        let key_schema_defintions = parse_key_schema_defintion_json(key_schema_json_path);
        let throughput = ProvisionedThroughput::builder()
            .read_capacity_units(10)
            .write_capacity_units(10)
            .build();
        let outputs = client
            .create_table()
            .table_name(table_name)
            .set_attribute_definitions(Some(attribute_definitions))
            .set_key_schema(Some(key_schema_defintions))
            .provisioned_throughput(throughput)
            .send()
            .await
            .expect("Error while creating DynamoDB Table\n");
        if let Some(table_description) = outputs.table_description {
            if let Some(time) = table_description.creation_date_time {
                let convert = time
                    .fmt(aws_sdk_dynamodb::primitives::DateTimeFormat::HttpDate)
                    .ok();
                if let Some(data_time) = convert {
                    println!("Creation Time Of the Table: {}", data_time.green().bold());
                }
            }
            if let Some(table_status) = table_description.table_status {
                let convert_to_str = table_status.as_str();
                println!(
                    "Current Status of '{}' table: {}",
                    table_name.green().bold(),
                    convert_to_str.green().bold()
                );
            }
            if let Some(table_size) = table_description.table_size_bytes {
                println!("Table Size In Bytes: {}", table_size);
            }
            if let Some(table_id) = table_description.table_id {
                println!("Table Identifier: {}", table_id.green().bold());
            }
            if let Some(table_arn) = table_description.table_arn {
                println!(
                    "Amazon Resource Name(ARN) for the Table: {}",
                    table_arn.green().bold()
                );
            }
            if let Some(item_counts) = table_description.item_count {
                println!(
                    "Number of Items in the Table: {}",
                    item_counts.to_string().green().bold()
                );
            }
            if let Some(delete_protection) = table_description.deletion_protection_enabled {
                let format = format!(
                    "whether deletion protection is enabled: {}",
                    delete_protection
                );
                println!("{}\n", format);
            }
        }
    }
}
/// ['AttributeDefinition`](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_AttributeDefinition.html)
fn parse_attribute_defintion_json(json_path: &str) -> Vec<AttributeDefinition> {
    let file = File::open(json_path).expect("Error while Opening the Json path you specified\n");
    let buffer = BufReader::new(file);
    let load_json_data: Value =
        serde_json::from_reader(buffer).expect("Error Parsing the Json File\n");
    let read_json = load_json_data.as_object();
    let mut vec_attribute_definitions = Vec::new();
    if let Some(json_data) = read_json {
        let attribute_names = json_data.keys();
        let attributes_types = json_data.values();
        for (att_name, att_type) in attribute_names.zip(attributes_types) {
            let att_type = att_type.as_str();
            if let Some(type_) = att_type {
                let scalar_type = ScalarAttributeType::from(type_);
                let attribute_definition_builder = AttributeDefinition::builder()
                    .attribute_name(att_name)
                    .attribute_type(scalar_type)
                    .build();
                vec_attribute_definitions.push(attribute_definition_builder);
            }
        }
    }
    vec_attribute_definitions
}

fn parse_key_schema_defintion_json(json_path: &str) -> Vec<KeySchemaElement> {
    let file = File::open(json_path)
        .expect("Error while opening key schema json the path you specified\n");
    let buffer = BufReader::new(file);
    let load_json_data: Value = serde_json::from_reader(buffer).expect("Error opening json File\n");
    let read_json = load_json_data.as_object();
    let mut vec_key_schema_definitions = Vec::new();
    if let Some(json_data) = read_json {
        let attribute_names = json_data.keys();
        let key_types = json_data.values();
        for (att_name, att_type) in attribute_names.zip(key_types) {
            let value_of_key_type = att_type.as_str();
            if let Some(type_) = value_of_key_type {
                let key_type = KeyType::from(type_);
                let key_schema_builder = KeySchemaElement::builder()
                    .attribute_name(att_name)
                    .key_type(key_type)
                    .build();
                vec_key_schema_definitions.push(key_schema_builder);
            }
        }
    }
    vec_key_schema_definitions
}
