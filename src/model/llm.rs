//! This module defines the data model for LLMs (Large Language Model), such as OpenAI GPT-3/4, etc. Also, it can use the LLM to answer the question.

use super::core::{Entity, RecordResponse, Relation};
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, FunctionCall, MessageRole};
use openai_api_rs::v1::common::{GPT3_5_TURBO, GPT4};
use poem_openapi::{Enum, Object};
use regex::Regex;
use log::{warn};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Object, sqlx::FromRow)]
pub struct LlmResponse {
    pub prompt: String,
    pub response: String,
    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
}

/// The context is used to store the context for the LLM. The context can be an entity, an expanded relation, or a symptoms with disease context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Context {
    pub entity: Option<Entity>,
    pub expanded_relation: Option<ExpandedRelation>,
    pub symptoms_with_disease_ctx: Option<SymptomsWithDiseaseCtx>,
}

impl Context {
    pub async fn answer(self, chatbot: &ChatBot, prompt_template_id: &str) -> Result<LlmResponse, anyhow::Error> {
        let resp = if self.entity.is_some() {
            let entity = self.entity.unwrap();
            let mut llm_msg = LlmMessage::new(&prompt_template_id, entity, None).unwrap();
            let answer = llm_msg.answer(&chatbot, None).await.unwrap();
            Ok(LlmResponse {
                prompt: answer.prompt.to_owned(),
                response: answer.message.to_owned(),
                created_at: answer.created_at,
            })
        } else if self.expanded_relation.is_some() {
            let expanded_relation = self.expanded_relation.unwrap();
            let mut llm_msg =
                LlmMessage::new(&prompt_template_id, expanded_relation, None).unwrap();
            let answer = llm_msg.answer(&chatbot, None).await.unwrap();
            Ok(LlmResponse {
                prompt: answer.prompt.to_owned(),
                response: answer.message.to_owned(),
                created_at: answer.created_at,
            })
        } else if self.symptoms_with_disease_ctx.is_some() {
            let symptoms_with_disease_ctx = self.symptoms_with_disease_ctx.unwrap();
            let mut llm_msg =
                LlmMessage::new(&prompt_template_id, symptoms_with_disease_ctx, None).unwrap();
            let answer = llm_msg.answer(&chatbot, None).await.unwrap();
            Ok(LlmResponse {
                prompt: answer.prompt.to_owned(),
                response: answer.message.to_owned(),
                created_at: answer.created_at,
            })
        } else {
            let err =
                "One of entity, expanded_relation or symptoms_with_disease_ctx must be provided."
                    .to_string();
            Err(anyhow::anyhow!(err))
        };

        return resp;
    }
}

/// A trait for LLM context. Each LLM context might can render several prompt templates. So we separate the context and prompt template. But this means the users need to provide the right pair of context and prompt template.
pub trait LlmContext {
    fn get_context(&self) -> Self;
    fn render_prompt(&self, prompt_template: &str) -> String;
}

impl LlmContext for Entity {
    fn get_context(&self) -> Self {
        self.clone()
    }

    fn render_prompt(&self, prompt_template: &str) -> String {
        let mut prompt = prompt_template.to_string();
        prompt = prompt.replace("{{entity_name}}", &self.name);
        prompt = prompt.replace("{{entity_id}}", &self.id);
        prompt = prompt.replace("{{entity_type}}", &self.label);
        prompt
    }
}

/// The expanded relation is used to store the relation between two entities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct ExpandedRelation {
    pub relation: Relation,
    pub source: Entity,
    pub target: Entity,
}

impl LlmContext for ExpandedRelation {
    fn get_context(&self) -> Self {
        self.clone()
    }

    fn render_prompt(&self, prompt_template: &str) -> String {
        let mut prompt = prompt_template.to_string();
        prompt = prompt.replace("{{source_name}}", &self.source.name);
        prompt = prompt.replace("{{source_id}}", &self.source.id);
        prompt = prompt.replace("{{source_type}}", &self.source.label);
        prompt = prompt.replace("{{relation_type}}", &self.relation.relation_type);
        prompt = prompt.replace("{{target_name}}", &self.target.name);
        prompt = prompt.replace("{{target_id}}", &self.target.id);
        prompt = prompt.replace("{{target_type}}", &self.target.label);
        prompt
    }
}

// The SymptomsWithDiseaseCtx is used to store the context for the symptoms with disease.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct SymptomsWithDiseaseCtx {
    pub disease_name: String,
    pub subgraph: String,
    pub symptoms: Vec<String>,
}

impl LlmContext for SymptomsWithDiseaseCtx {
    fn get_context(&self) -> Self {
        self.clone()
    }

    fn render_prompt(&self, prompt_template: &str) -> String {
        let mut prompt = prompt_template.to_string();
        prompt = prompt.replace("{{disease_name}}", &self.disease_name);
        prompt = prompt.replace("{{subgraph}}", &self.subgraph);
        prompt = prompt.replace("{{symptoms}}", &self.symptoms.join(", "));
        prompt
    }
}

lazy_static! {
    pub static ref UUID_REGEX: Regex =
        Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();
    // Only for predicted edge
    pub static ref PROMPT_TEMPLATE: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        m.insert("node_summary", "You need to execute the following instructions I send you: find the related information for the question, summarize the information you found and output a summary no more than 500 words, give me the sources of information. Notice: Please just return me the sentence 'I don't know what you say, it seems not to be a right question related with specific topic', if the question I send you is not related with medical concepts, such as {{entity_type}}.\n\nWhat's the {{entity_name}} which id is {{entity_id}}?");

        m.insert("edge_summary", "You need to execute the following instructions I send you: find the related information for the question, summarize the information you found and output a summary no more than 500 words, give me the sources of information. Notice: Please just return me the sentence 'I don't know what you say, it seems not to be a right question related with specific topic', if the question I send you is not related with medical concepts.\n\nWhat's the {{source_name}}[{{source_id}}, {{source_type}}] -> {{relation_type}} -> {{target_name}}[{{target_id}}, {{target_type}}?");

        m.insert("custom_question", "You need to execute the following instructions I send you: find the related information for the question, summarize the information you found and output a summary no more than 500 words, give me the sources of information. Notice: Please just return me the sentence 'I don't know what you say, it seems not to be a right question related with specific topic', if the question I send you is not related with medical concepts.\n\n{{custom_question}}");

        // You need to prepare two fields: 1) subgraph: a json string; 2) disease_name: a string.
        m.insert("subgraph_symptoms_with_disease_ctx", "Knowledge Subgraph: {{subgraph}}\n\nQuestions:\nI have a new Knowledge Subgraph focusing on {{disease_name}} that encompasses related symptoms, diseases, medications, and genes/pathways. This Subgraph includes: 1) Associations between {{disease_name}} related symptoms and various diseases; 2) Medications and genes/pathways related to these diseases; 3) How medications exert therapeutic effects through specific genes or pathways for these diseases. 4) {{disease_name}} related symptoms: {{symptoms}}. I am looking to understand:\n1) Which diseases are directly associated with {{disease_name}} symptoms and the common treatment medications for these diseases.\n2) How these medications work by affecting certain genes or pathways.3) If there are any new studies or predictive relationships indicating unrecognized medications that could be beneficial for {{disease_name}} symptoms.\n\nPlease answer the above questions according to the knowledge subgraph and your knowledges.");

        m
    };
}

/// The prompt template category is used to store the category of prompt template. Each category has a prompt template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Enum)]
pub enum PromptTemplateCategoryEnum {
    NodeSummary,
    EdgeSummary,
    CustomQuestion,
    SubgraphSymptomsWithDiseaseCtx,
}

/// A wrapper for prompt template category.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct PromptTemplateCategory {
    value: PromptTemplateCategoryEnum,
}

/// Implement the conversion between PromptTemplateCategory and PromptTemplateCategoryEnum.
impl From<String> for PromptTemplateCategory {
    fn from(v: String) -> Self {
        match v.as_str() {
            "node_summary" => PromptTemplateCategory {
                value: PromptTemplateCategoryEnum::NodeSummary,
            },
            "edge_summary" => PromptTemplateCategory {
                value: PromptTemplateCategoryEnum::EdgeSummary,
            },
            "custom_question" => PromptTemplateCategory {
                value: PromptTemplateCategoryEnum::CustomQuestion,
            },
            "subgraph_symptoms_with_disease_ctx" => PromptTemplateCategory {
                value: PromptTemplateCategoryEnum::SubgraphSymptomsWithDiseaseCtx,
            },
            _ => panic!("Invalid prompt template category"),
        }
    }
}

/// Implement the conversion between PromptTemplateCategory and PromptTemplateCategoryEnum.
impl Into<PromptTemplateCategoryEnum> for PromptTemplateCategory {
    fn into(self) -> PromptTemplateCategoryEnum {
        self.value
    }
}

/// Implement the conversion between PromptTemplateCategory and String.
impl Into<String> for PromptTemplateCategory {
    fn into(self) -> String {
        match self.value {
            PromptTemplateCategoryEnum::NodeSummary => "node_summary".to_string(),
            PromptTemplateCategoryEnum::EdgeSummary => "edge_summary".to_string(),
            PromptTemplateCategoryEnum::CustomQuestion => "custom_question".to_string(),
            PromptTemplateCategoryEnum::SubgraphSymptomsWithDiseaseCtx => {
                "subgraph_symptoms_with_disease_ctx".to_string()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow, Object, Validate)]
pub struct LlmMessage<
    T: LlmContext
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
> {
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    pub id: i32,

    #[validate(regex(
        path = "UUID_REGEX",
        message = "The session_id must match the ^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$ pattern."
    ))]
    pub session_uuid: String,
    pub prompt_template: String,
    pub prompt_template_category: PromptTemplateCategory,

    pub context: T,
    pub prompt: String,
    pub message: String,

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    pub created_at: DateTime<Utc>,

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    pub updated_at: DateTime<Utc>,
}

impl<T: LlmContext> LlmMessage<T>
where
    T: LlmContext
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
{
    pub fn new(
        // TODO: User how to know the right prompt template category for a given context?
        prompt_template_category: &str,
        context: T,
        session_uuid: Option<String>,
    ) -> Result<Self, anyhow::Error> {
        let session_uuid = match session_uuid {
            Some(session_uuid) => session_uuid,
            None => uuid::Uuid::new_v4().to_string(),
        };

        let prompt_template = match PROMPT_TEMPLATE.get(prompt_template_category) {
            Some(prompt_template) => prompt_template.to_string(),
            None => return Err(anyhow::anyhow!("Invalid prompt template category")),
        };

        let prompt = context.render_prompt(prompt_template.as_str());
        let message = "".to_string();

        let prompt_template_category = PromptTemplateCategory::from(prompt_template_category.to_string());

        Ok(LlmMessage {
            id: 0,
            session_uuid,
            prompt_template,
            prompt_template_category,
            context,
            prompt,
            message,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn save2db(&self, pool: &sqlx::PgPool) -> Result<&Self, anyhow::Error> {
        let sql_str = "INSERT INTO biomedgps_ai_message (session_uuid, prompt_template, prompt_template_category, context, prompt, message, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id";

        let prompt_template_category: String = self.prompt_template_category.clone().into();
        let context: String = self.context.to_json_string();
        let query = sqlx::query(&sql_str)
            .bind(&self.session_uuid)
            .bind(&self.prompt_template)
            .bind(&prompt_template_category)
            .bind(&context)
            .bind(&self.prompt)
            .bind(&self.message)
            .bind(&self.created_at)
            .bind(&self.updated_at);

        match query.execute(pool).await {
            Ok(_) => return Ok(self),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to save message to database: {}",
                    e.to_string()
                ))
            }
        };
    }

    pub async fn answer(&mut self, chatbot: &ChatBot, pool: Option<&sqlx::PgPool>) -> Result<&Self, anyhow::Error> {
        let prompt = self.prompt.clone();
        self.message = chatbot.answer(prompt)?;
        self.updated_at = Utc::now();

        if pool.is_none() {
            return Ok(self);
        }

        match self.save2db(pool.unwrap()).await {
            Ok(_) => {
                Ok(self)
            }
            Err(e) => Err(anyhow::anyhow!(
                "Failed to save message to database: {}",
                e.to_string()
            )),
        }
    }
}

pub struct ChatBot {
    role: MessageRole,
    name: Option<String>,
    content: Option<String>,
    function_call: Option<FunctionCall>,
    model_name: String,
    client: Client,
}

impl ChatBot {
    pub fn new(model_name: &str, openai_api_key: &str) -> Self {
        let model = if model_name == "GPT4" {
            GPT4.to_string()
        } else {
            GPT3_5_TURBO.to_string()
        };

        let client = Client::new(openai_api_key.to_string());

        ChatBot {
            role: MessageRole::user,
            name: None,
            content: None,
            function_call: None,
            model_name: model,
            client: client,
        }
    }

    pub fn answer(&self, prompt: String) -> Result<String, anyhow::Error> {
        let model_name = self.model_name.clone();
        let req = ChatCompletionRequest::new(
            model_name,
            vec![chat_completion::ChatCompletionMessage {
                role: self.role.clone(),
                content: prompt,
                name: self.name.clone(),
                function_call: self.function_call.clone(),
            }],
        );

        let result = self.client.chat_completion(req)?;
        let message = result.choices[0].message.content.clone();

        match message {
            Some(message) => Ok(message),
            None => Err(anyhow::anyhow!("No message returned")),
        }
    }
}


// Write unit tests
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_answer() {
        let OPENAI_API_KEY = std::env::var("OPENAI_API_KEY").unwrap();
        let chatbot = super::ChatBot::new("GPT3", &OPENAI_API_KEY);

        let node = super::Entity {
            idx: 0,
            id: "DrugBank:DB01050".to_string(),
            name: "IBUPROFEN".to_string(),
            label: "Compound".to_string(),
            resource: "DrugBank".to_string(),
            description: None,
            taxid: None,
            synonyms: None,
            pmids: None,
            xrefs: None,
        };

        let mut llm_msg = super::LlmMessage::new("node_summary", node, None).unwrap();
        let answer = llm_msg.answer(&chatbot, None).await.unwrap();
        println!("Prompt: {}", answer.prompt);
        println!("Answer: {}", answer.message);

        assert_eq!(answer.message.len() > 0, true);
    }
}