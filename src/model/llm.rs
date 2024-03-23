//! This module defines the data model for LLMs (Large Language Model), such as OpenAI GPT-3/4, etc. Also, it can use the LLM to answer the question.

use super::core::{Entity, Relation};
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::warn;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, FunctionCall, MessageRole};
use openai_api_rs::v1::common::{GPT3_5_TURBO, GPT4, GPT4_1106_PREVIEW};
use openssl::hash::{hash, MessageDigest};
use poem_openapi::{Enum, Object};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::sync::Mutex;
use validator::Validate;

lazy_static! {
    pub static ref UUID_REGEX: Regex =
        Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();

    pub static ref PROMPTS: Mutex<Vec<HashMap<&'static str, &'static str>>> = Mutex::new(vec![]);
    pub static ref PROMPT_TEMPLATE: Mutex<HashMap<&'static str, &'static str>> = Mutex::new(HashMap::new());
    // "explain_custom_question": "You need to execute the following instructions I send you: find the related information for the question, summarize the information you found and output a summary no more than 500 words, give me the sources of information. Notice: Please just return me the sentence 'I don't know what you say, it seems not to be a right question related with specific topic', if the question I send you is not related with medical concepts.\n\n{{custom_question}}"
}

// Initialize the prompt templates
pub fn init_prompt_templates() {
    // Register the prompt templates for the entity
    Entity::register_prompt_template();

    // Register the prompt templates for the expanded relation
    ExpandedRelation::register_prompt_template();

    // Register the prompt templates for the subgraph with disease context
    SubgraphWithCtx::register_prompt_template();
}

#[derive(Debug, Deserialize, Serialize, Object, sqlx::FromRow)]
pub struct LlmResponse {
    pub prompt: String,
    pub response: String,
    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
}

/// The context is used to store the context for the LLM. The context can be an entity, an expanded relation, or treatments with disease context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Context {
    pub entity: Option<Entity>,
    pub expanded_relation: Option<ExpandedRelation>,
    pub subgraph_with_ctx: Option<SubgraphWithCtx>,
}

impl Context {
    pub async fn answer(
        self,
        chatbot: &ChatBot,
        prompt_template_id: &str,
        pool: Option<&sqlx::PgPool>,
    ) -> Result<LlmResponse, anyhow::Error> {
        let resp = if self.entity.is_some() {
            let entity = self.entity.unwrap();
            let mut llm_msg = LlmMessage::new(&prompt_template_id, entity, None).unwrap();
            let answer = llm_msg.answer(&chatbot, pool).await.unwrap();
            Ok(LlmResponse {
                prompt: answer.prompt.to_owned(),
                response: answer.message.to_owned(),
                created_at: answer.created_at,
            })
        } else if self.expanded_relation.is_some() {
            let expanded_relation = self.expanded_relation.unwrap();
            let mut llm_msg =
                LlmMessage::new(&prompt_template_id, expanded_relation, None).unwrap();
            let answer = llm_msg.answer(&chatbot, pool).await.unwrap();
            Ok(LlmResponse {
                prompt: answer.prompt.to_owned(),
                response: answer.message.to_owned(),
                created_at: answer.created_at,
            })
        } else if self.subgraph_with_ctx.is_some() {
            let subgraph_with_ctx = self.subgraph_with_ctx.unwrap();
            let mut llm_msg =
                LlmMessage::new(&prompt_template_id, subgraph_with_ctx, None).unwrap();
            let answer = match llm_msg.answer(&chatbot, pool).await {
                Ok(answer) => answer,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to answer the question: {}",
                        e.to_string()
                    ));
                }
            };
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
    fn render_prompt(&self, prompt_template_category: &str, prompt_template: &str) -> Result<String, anyhow::Error>;
    fn register_prompt_template();
}

impl LlmContext for Entity {
    fn get_context(&self) -> Self {
        self.clone()
    }

    fn render_prompt(&self, prompt_template_category: &str, prompt_template: &str) -> Result<String, anyhow::Error> {
        let mut prompt = prompt_template.to_string();
        prompt = prompt.replace("{{entity_name}}", &self.name);
        prompt = prompt.replace("{{entity_id}}", &self.id);
        prompt = prompt.replace("{{entity_type}}", &self.label);
        Ok(prompt)
    }

    fn register_prompt_template() {
        let mut prompt_templates = PROMPT_TEMPLATE.lock().unwrap();
        prompt_templates.insert("explain_node_summary", "You need to execute the following instructions I send you: find the related information for the question, summarize the information you found and output a summary no more than 500 words, give me the sources of information. \n\nWhat's the {{entity_name}} which id is {{entity_id}}?");

        let mut prompts = PROMPTS.lock().unwrap();

        let mut m1 = HashMap::new();
        m1.insert("key", "explain_node_summary");
        m1.insert("label", "Node Summary");
        m1.insert("type", "node");

        // Does it exist?
        if prompts.contains(&m1) {
            return;
        } else {
            prompts.push(m1);
        }
    }
}

/// The expanded relation is used to store the relation between two entities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow)]
pub struct ExpandedRelation {
    pub relation: Relation,
    pub source: Entity,
    pub target: Entity,
}

impl LlmContext for ExpandedRelation {
    fn get_context(&self) -> Self {
        self.clone()
    }

    fn render_prompt(&self, prompt_template_category: &str, prompt_template: &str) -> Result<String, anyhow::Error> {
        let mut prompt = prompt_template.to_string();
        prompt = prompt.replace("{{source_name}}", &self.source.name);
        prompt = prompt.replace("{{source_id}}", &self.source.id);
        prompt = prompt.replace("{{source_type}}", &self.source.label);
        prompt = prompt.replace("{{relation_type}}", &self.relation.relation_type);
        prompt = prompt.replace("{{target_name}}", &self.target.name);
        prompt = prompt.replace("{{target_id}}", &self.target.id);
        prompt = prompt.replace("{{target_type}}", &self.target.label);
        Ok(prompt)
    }

    fn register_prompt_template() {
        let mut prompt_templates = PROMPT_TEMPLATE.lock().unwrap();
        prompt_templates.insert("explain_edge_summary", "You need to execute the following instructions I send you: find the related information for the question, summarize the information you found and output a summary no more than 500 words, give me the sources of information. What's the {{source_name}}[{{source_id}}, {{source_type}}] -> {{relation_type}} -> {{target_name}}[{{target_id}}, {{target_type}}? Do you know more about the relationship between {{source_name}} and {{target_name}}? If you know, please tell me more about the relationship between {{source_name}} and {{target_name}}.");

        let mut prompts = PROMPTS.lock().unwrap();

        let mut m2 = HashMap::new();
        m2.insert("key", "explain_edge_summary");
        m2.insert("label", "Edge Summary");
        m2.insert("type", "edge");

        // Does it exist?
        if prompts.contains(&m2) {
            return;
        } else {
            prompts.push(m2);
        }
    }
}

// The SubgraphWithCtx is used to store the context and a related subgraph for explaining.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow)]
pub struct SubgraphWithCtx {
    pub context_str: String,
    pub subgraph: String,
}

impl LlmContext for SubgraphWithCtx {
    fn get_context(&self) -> Self {
        self.clone()
    }

    fn render_prompt(&self, prompt_template_category: &str, prompt_template: &str) -> Result<String, anyhow::Error> {
        let ctx_str_regex = Regex::new(r"(.*)#(.*)#(.*)").unwrap();
        let err_msg = "Invalid context_str, it should be a combination of a node1, a relation type, and a node2, such as `Ibuprofen#treats#Headache` when you use the prompt template `explain_path_with_attention_subgraph`".to_string();
        if prompt_template_category == "explain_path_with_attention_subgraph" {
            let caps = match ctx_str_regex.captures(&self.context_str) {
                Some(caps) => caps,
                None => return Err(anyhow::anyhow!(err_msg)),
            };
            let node1 = match caps.get(1) {
                Some(node1) => node1.as_str(),
                None => return Err(anyhow::anyhow!(err_msg)),
            };
            let relation_type = match caps.get(2) {
                Some(relation_type) => relation_type.as_str(),
                None => return Err(anyhow::anyhow!(err_msg)),
            };
            let node2 = match caps.get(3) {
                Some(node2) => node2.as_str(),
                None => return Err(anyhow::anyhow!(err_msg)),
            };

            let mut prompt = prompt_template.to_string();
            prompt = prompt.replace("{{subgraph}}", &self.subgraph);
            prompt = prompt.replace("{{node1}}", node1);
            prompt = prompt.replace("{{relation_type}}", relation_type);
            prompt = prompt.replace("{{node2}}", node2);
            return Ok(prompt);
        } else {
            let mut prompt = prompt_template.to_string();
            prompt = prompt.replace("{{context_str}}", &self.context_str);
            prompt = prompt.replace("{{subgraph}}", &self.subgraph);
            return Ok(prompt);
        }
    }

    fn register_prompt_template() {
        let mut prompts = PROMPTS.lock().unwrap();
        let mut prompt_templates = PROMPT_TEMPLATE.lock().unwrap();

        let mut m1 = HashMap::new();
        m1.insert("key", "explain_subgraph_treatment_with_disease_ctx");
        m1.insert("label", "Treatment within Disease Context");
        m1.insert("type", "subgraph");

        // Does it exist?
        if prompts.contains(&m1) {
            return;
        } else {
            prompts.push(m1);
        };
        // You need to prepare two fields: 1) subgraph: a json string; 2) context_str: a string, it need to be a disease name, such as "ME/CFS".
        prompt_templates.insert("explain_subgraph_treatment_with_disease_ctx", "Knowledge Subgraph: {{subgraph}}\n\nKnowledge Subgraph Analysis Request:\n\nSubgraph Overview: I have compiled a Knowledge Subgraph dedicated to exploring the complex landscape surrounding {{context_str}}, incorporating elements such as related symptoms, co-occurring diseases, therapeutic medications, and underlying genes/pathways. This Subgraph aims to elucidate:\n\nDisease-Symptom Associations: The linkages between symptoms of {{context_str}} and their correlation with various diseases.\nMedication and Genetic/Pathway Connections: How medications align with and influence the genes or pathways associated with these diseases.\nMechanisms of Action: The specific pathways through which medications exert their therapeutic effects on these diseases.\nSymptom Detailing for {{context_str}}: Specific symptoms related to {{context_str}}.\nResearch Questions:\n\nIn light of the above, my queries are as follows:\n\nCritical Knowledge Identification: Within the context of {{context_str}}, this Knowledge Subgraph houses an extensive array of entities and relationships fundamental to unraveling the disease's mechanisms and scrutinizing relevant treatment drugs. Leveraging your expertise, concentrate on the graph's relations pivotal to understanding {{context_str}}'s mechanisms and its associated treatments. Identify and emphasize essential knowledge aspects that significantly aid in decoding the disease's pathology and therapeutic measures. This entails pinpointing vital biological pathways, gene-disease correlations, drug-target engagements, and any novel research insights that could reveal innovative therapeutic approaches. Your analysis is expected to prioritize data with a direct bearing on treatment efficacy and enhance our molecular-level understanding of the disease.\n\nEmerging Therapies: Are there any novel studies or predictive analyses indicating unrecognized medications that might benefit {{context_str}} symptoms or the disease itself?\n\nSymptom-Disease Correlation: Which diseases are directly linked to {{context_str}} symptoms, and what are the common treatments for these diseases?\n\nAction Mechanisms of Medications: How do these medications influence specific genes or pathways?\n\nGuidance for Response:\n\nPlease address the aforementioned inquiries based on the Knowledge Subgraph and your expertise. For each of the questions related to the Knowledge Subgraph and its implications for {{context_str}}, it is imperative that you provide supporting literature. This literature must exclusively come from PubMed, which is a critical repository for reliable medical research findings. Your responses should not only incorporate insights derived from these studies but also include citations formatted according to standard academic practices. Specifically, citations should detail the authors, title, journal name, year of publication, and the PubMed ID (PMID) to facilitate easy verification and further reading.\n\nFor example, a proper citation format would be: Doe J, Smith A, Jones B. Title of the Article. Journal Name. Year;Volume(Issue):Page numbers. PMID: XXXXXXX.\nThis requirement is non-negotiable, ensuring that all information provided is backed by credible and accessible scientific evidence. Leveraging PubMed as a source is essential for maintaining the accuracy and reliability of the insights shared in your analysis.");

        let mut m2 = HashMap::new();
        m2.insert("key", "explain_subgraph_mechanism_with_disease_ctx");
        m2.insert("label", "Mechanism within Disease Context");
        m2.insert("type", "subgraph");

        // Does it exist?
        if prompts.contains(&m2) {
            return;
        } else {
            prompts.push(m2);
        };

        // You need to prepare two fields: 1) subgraph: a json string; 2) context_str: a string, it need to be a disease name, such as "ME/CFS".
        prompt_templates.insert("explain_subgraph_mechanism_with_disease_ctx", "Knowledge Subgraph: {{subgraph}}\n\nKnowledge Subgraph Analysis Request:\nI have a set of Subgraph data that includes a collection of genes/proteins and their connections to a specific disease, {{context_str}}. This Subgraph consists of nodes (representing genes/proteins) and edges (representing interactions or relationships between the genes/proteins). Each node has associated attributes, such as name, description, or known disease associations. Edges may also have attributes, like the type or strength of interaction. My goal is to identify key nodes and paths within this Subgraph that are most relevant to {{context_str}}. To achieve this, I need your assistance to: 1. Identify and explain the key nodes that are most relevant to {{context_str}}. Please base your explanation on the nodes and their attributes and their known roles in the disease, explaining which nodes are critical and why these nodes are critical. 2. Determine and describe the main paths connecting these key nodes. Please discuss how these paths might be involved in the onset, progression, or treatment of the disease, considering the type and strength of interactions between nodes. 3. Provide a report summarizing your findings and understanding, including a list of key nodes and paths, along with a rationale for how they are related to {{context_str}}. Note that, given the complexity and multifactorial nature of diseases, explanations may need to integrate multiple attributes of nodes and their interactions. \n\nGuidance for Response:\n\nPlease address the aforementioned inquiries based on the Knowledge Subgraph and your expertise. For each of the questions related to the Knowledge Subgraph and its implications for {{context_str}}, it is imperative that you provide supporting literature. This literature must exclusively come from PubMed, which is a critical repository for reliable medical research findings. Your responses should not only incorporate insights derived from these studies but also include citations formatted according to standard academic practices. Specifically, citations should detail the authors, title, journal name, year of publication, and the PubMed ID (PMID) to facilitate easy verification and further reading.\n\nFor example, a proper citation format would be: Doe J, Smith A, Jones B. Title of the Article. Journal Name. Year;Volume(Issue):Page numbers. PMID: XXXXXXX.");

        let mut m3 = HashMap::new();
        m3.insert("key", "explain_subgraph_importance_with_disease_ctx");
        m3.insert("label", "Node/Edge Importance within Disease Context");
        m3.insert("type", "subgraph");

        // Does it exist?
        if prompts.contains(&m3) {
            return;
        } else {
            prompts.push(m3);
        };
        // JSON version
        // You need to prepare two fields: 1) subgraph: a json string; 2) context_str: a string, it need to be a disease name, such as "ME/CFS".
        // m.insert("explain_subgraph_importance_with_disease_ctx", "Knowledge Subgraph: {{subgraph}}\n\nKnowledge Subgraph Analysis Request:\nI have a set of Subgraph data that includes a collection of genes/proteins and their connections to a specific disease, {{context_str}}. This Subgraph consists of nodes (representing genes/proteins) and edges (representing interactions or relationships between the genes/proteins). Each node has associated attributes, such as name, description, or known disease associations. Edges may also have attributes, like the type or strength of interaction. My goal is to identify key nodes and paths within this Subgraph that are most relevant to {{context_str}}. To achieve this, please label these nodes and paths as critical, important, moderate, or less important based on your knowledges and the subgraph, and provide a rationale for your assessment. After labeling the nodes and paths, please output your findings as an array, the format is like```{your_output}```.  The array contains a set of objects, each object have as least three columns: id (node or edge), importance, reason.");

        // Table version
        prompt_templates.insert("explain_subgraph_importance_with_disease_ctx", "Knowledge Subgraph: {{subgraph}}\n\nKnowledge Subgraph Analysis Request:\nI have a set of Subgraph data that includes a collection of genes/proteins and their connections to a specific disease, {{context_str}}. This Subgraph consists of nodes (representing genes/proteins) and edges (representing interactions or relationships between the genes/proteins). Each node has associated attributes, such as name, description, or known disease associations. Edges may also have attributes, like the type or strength of interaction. My goal is to identify key nodes and edges within this Subgraph that are most relevant to {{context_str}}. To achieve this, please label these nodes listed in the subgraph as Critical, Important, Moderate, or Less Important based on your knowledges on {{context_str}} and the subgraph, and provide a rationale for your assessment. After labeling the nodes, please output your results as a table (not a file). The table contains a set of rows, each row have as least six columns: #, ID (node id), Name (node name), Importance, Reliability, Reason. Please note: 1. the subgraph might be incomplete, so you need to use your knowledges to think these nodes step by step, and then assess the importance of the nodes; 2. you need to consider the importance of the node types for specific diseases, for example, symptom might be important for a symptom-defined disease, but it might be less important for a genetic disease; 3. you need to consider the reliability of the relation, for example, if the relation is mentioned more frequent in your knowledgebase, then we can treat it more reliable; 4. the final table you output should ordered by importance and reliability, and the importance should be ordered by Critical, Important, Moderate, and Less Important; 5. you need to tell me why you think the node is important / less important, reliable / less reliable, and the reason should be based on the subgraph and your knowledges (recommendation).");

        let mut m4 = HashMap::new();
        m4.insert("key", "explain_path_within_subgraph");
        m4.insert("label", "Path within Subgraph");
        m4.insert("type", "path");

        // Does it exist?
        if prompts.contains(&m4) {
            return;
        } else {
            prompts.push(m4);
        };
        // Actually, in this case, the context_str is a path name, such as "ME/CFS-[treated_by]->Ibuprofen-[treats]->Headache".
        prompt_templates.insert("explain_path_within_subgraph", "Knowledge Subgraph: {{subgraph}}\n\nMy goal is to explain the path {{context_str}} within the subgraph. Please provide a detailed explanation of the path.");

        let mut m5 = HashMap::new();
        m5.insert("key", "explain_path_with_attention_subgraph");
        m5.insert("label", "Path with Attention Subgraph");
        m5.insert("type", "edge");

        // Does it exist?
        if prompts.contains(&m5) {
            return;
        } else {
            prompts.push(m5);
        };

        // In this case, the context_str is a combination of a node1, a relation type, and a node2, such as "Ibuprofen-[treats]->Headache". So we split the context_str into three parts: node1, relation_type, and node2 for rendering the prompt.
        prompt_templates.insert("explain_path_with_attention_subgraph", "Knowledge Subgraph: {{subgraph}}\n\nGiven a Subgraph extracted from a Knowledge Graph, the Subgraph is centered around two main nodes, {{node1}} and {{node2}}, and it is structured to provide context for the existence of a specific relationship {{relation_type}} between them. The Subgraph includes the following elements: 1. Direct Relationship: Information on whether there is a direct connection of type {{relation_type}} between {{node1}} and {{node2}}. 2. {{node1}}'s Top Connections: A list of the top 10 connections for node {{node1}}, including the type of relationship and the connected nodes, ranked by score indicating the strength and relevance of each connection. 3. {{node2}}'s Top Connections: A list of the top 10 connections for node {{node2}}, similar to {{node1}}'s connections, providing insights into {{node2}}'s most significant relationships in the Knowledge Graph. 4. Relevant Attributes of Nodes: Any relevant attributes or properties of {{node1}}, {{node2}}, and their top connected nodes that could influence the presence of relationship {{relation_type}}.\n\nAdditional Context: Any other nodes or relationships within the Subgraph that provide further context or clues as to why relationship {{relation_type}} between {{node1}} and {{node2}} might be justified or logical. Based on the details of this Subgraph, please analyze and explain the potential reasons and the rationality behind the existence of relationship {{relation_type}} between node {{node1}} and node {{node2}}. Consider the direct and indirect connections, the significance of the relationships based on scores, and any attributes or contextual information provided.");
    }
}

// The following codes do not need to be modified when you add a new LLM context.
pub async fn fetch_by_session_uuid(
    session_uuid: &str,
    pool: &sqlx::PgPool,
) -> Result<LlmResponse, anyhow::Error> {
    let sql_str = "SELECT prompt, message as response, created_at FROM biomedgps_ai_message WHERE session_uuid = $1";
    match sqlx::query_as::<_, LlmResponse>(sql_str)
        .bind(session_uuid)
        .fetch_one(pool)
        .await
    {
        Ok(llm_response) => Ok(llm_response),
        Err(e) => Err(anyhow::anyhow!(
            "Failed to fetch the message by session_uuid: {}",
            e.to_string()
        )),
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
    pub prompt_template_category: String,

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
        + poem_openapi::types::ToJSON
        + Serialize,
{
    pub fn new(
        // TODO: User how to know the right prompt template category for a given context?
        prompt_template_category: &str,
        context: T,
        session_uuid: Option<String>,
    ) -> Result<Self, anyhow::Error> {
        let templates = PROMPT_TEMPLATE.lock().unwrap();
        let prompt_template = match templates.get(prompt_template_category) {
            Some(prompt_template) => prompt_template.to_string(),
            None => return Err(anyhow::anyhow!("Invalid prompt template category")),
        };

        let prompt = context.render_prompt(prompt_template_category, prompt_template.as_str())?;
        let session_uuid = match session_uuid {
            Some(session_uuid) => session_uuid,
            None => {
                let md5sum = hash(MessageDigest::md5(), prompt.as_bytes()).unwrap();
                let md5sum_uuid = uuid::Uuid::from_slice(&md5sum).unwrap();
                md5sum_uuid.to_string()
            }
        };
        let message = "".to_string();

        let prompt_template_category = if templates.contains_key(prompt_template_category) {
            prompt_template_category.to_string()
        } else {
            return Err(anyhow::anyhow!("Invalid prompt template category"));
        };

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

    pub async fn answer(
        &mut self,
        chatbot: &ChatBot,
        pool: Option<&sqlx::PgPool>,
    ) -> Result<&Self, anyhow::Error> {
        let prompt = self.prompt.clone();
        self.message = if pool.is_some() {
            match fetch_by_session_uuid(&self.session_uuid, pool.unwrap()).await {
                Ok(llm_response) => llm_response.response,
                Err(_) => "".to_string(),
            }
        } else {
            "".to_string()
        };

        if self.message.len() > 0 {
            return Ok(self);
        } else {
            self.message = match chatbot.answer(prompt) {
                Ok(message) => message,
                Err(e) => {
                    warn!("Failed to answer the question: {}", e.to_string());
                    return Err(anyhow::anyhow!(
                        "Failed to answer the question: {}",
                        e.to_string()
                    ));
                }
            };

            self.updated_at = Utc::now();

            if pool.is_some() {
                match self.save2db(pool.unwrap()).await {
                    Ok(_) => return Ok(self),
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "Failed to save message to database: {}",
                            e.to_string()
                        ))
                    }
                }
            } else {
                return Ok(self);
            }
        };
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
            // GPT4 or GPT4_1106_PREVIEW
            // https://platform.openai.com/account/limits
            //
            GPT4_1106_PREVIEW.to_string()
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

        let req = req.temperature(0.5);
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
