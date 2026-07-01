use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{llm::response::ResponseMessage, tool_call::tool::ToolCall};

/// LLM 请求载荷 + 消息历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    model: String,
    messages: Vec<Message>,
    thinking: Value,
    reasoning_effort: String,
    stream: bool,
    tools: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "system")]
    System { content: String },

    #[serde(rename = "assistant")]
    Assistant {
        content: String,
        reasoning_content: String,
        tool_calls: Option<Vec<ToolCall>>,
    },

    #[serde(rename = "user")]
    User { content: String },

    #[serde(rename = "tool")]
    Tool {
        content: String,
        tool_call_id: String,
    },
}

impl Conversation {
    pub fn new(model: String, tool_schemas: Vec<Value>) -> Self {
        Self {
            model,
            messages: Vec::new(),
            thinking: json!({"type": "enabled"}),
            reasoning_effort: "high".to_string(),
            stream: false,
            tools: tool_schemas,
        }
    }

    pub fn push_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn push_system(&mut self, content: String) {
        self.push_message(Message::System { content });
    }

    pub fn push_user(&mut self, content: String) {
        self.push_message(Message::User { content });
    }

    pub fn push_assistant(&mut self, msg: ResponseMessage) {
        self.push_message(Message::Assistant {
            content: msg.content,
            reasoning_content: msg.reasoning_content,
            tool_calls: msg.tool_calls,
        });
    }

    pub fn push_tool_result(&mut self, tool_call_id: String, content: String) {
        self.push_message(Message::Tool {
            content,
            tool_call_id,
        });
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let begin = ">".repeat(40);
        let end = "<".repeat(40);

        match self {
            Message::System { content } => {
                write!(f, "{}\nsystem:\n{}\n{}\n\n", begin, content, end)
            }

            Message::Assistant {
                content,
                reasoning_content,
                ..
            } => write!(
                f,
                "{}\nreasoning_content:\n{}\nassistant:\n{}\n{}\n\n",
                begin, reasoning_content, content, end
            ),

            Message::User { content, .. } => {
                write!(f, "{}\nuser:\n{}\n{}\n\n", begin, content, end)
            }

            Message::Tool {
                content: _content,
                tool_call_id,
            } => {
                write!(
                    f,
                    "{}\ntool({}):\n{}\n{}\n\n",
                    begin, tool_call_id, "muted now", end
                )
            }
        }
    }
}
