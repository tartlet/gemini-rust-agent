use reqwest::Client;
use serde_json::json;
use std::env;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct InteractionResponse {
    id: String,
    model: String,
    status: String,
    steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
struct Step {
    #[serde(rename = "type")]
    step_type: String,

    // a missing "content" field becomes Vec::new()
    #[serde(default)]
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,

    text: Option<String>,
}

struct Agent {
    api_key: String,
    url: String,
    client: Client,
    previous_interaction_id: Option<String>
}

impl Agent {
    fn new(url: String, api_key: String) -> Self {
        Self {
            url: url,
            client: Client::new(),
            api_key: api_key.to_string(),
            previous_interaction_id: None
        }
    }

    async fn ask(&mut self, input: &str) -> Result<String, Box<dyn std::error::Error>>  {
        let mut request_body = json!({
            "model": "gemini-3.7-flash",
            "input": &input
        });

        // look inside previous_interaction_id. If it contains a value, give me that value and call it id
        if let Some(id) = &self.previous_interaction_id {
            request_body["previous_interaction_id"] = json!(id);
        }

        let response = self.client
            .post(&self.url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        let body: InteractionResponse = response.json().await?;
        self.previous_interaction_id = Some(body.id.clone());

        if let Some(text) = extract_text(&body) {
            return Ok(text.to_string());
        }
        
        Err("Gemini response contained no text".into())
    }

}

fn get_user_input() -> String {
    let mut input = String::new();
    print!("Type something ... \n");

    use std::io::{self, Write};
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut input);

    let input: &str = input.trim();
    return input.to_string();
}

fn extract_text(response: &InteractionResponse) -> Option<&str> {
    // response
    //     .steps
    //     .iter()
    //     .filter(|step| step.step_type == "model_output")
    //     .flat_map(|step| &step.content)
    //     .find_map(|content| {
    //         if content.content_type == "text" {
    //             content.text.as_deref()
    //         } else {
    //             None
    //         }
    //     })
    for step in &response.steps {
        if step.step_type != "model_output" {
            continue
        }

        for content in &step.content {
            if content.content_type != "text" {
                continue
            }

            if let Some(text) = &content.text {
                return Some(text);
            }
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY")?;

    let mut agent = Agent::new(
        String::from("https://generativelanguage.googleapis.com/v1beta/interactions"), 
        api_key);

    loop {
        let input = get_user_input();

        if input == "quit" {
            break;
        }

        let response = agent.ask(&input).await?;
        println!("Gemini: {response}");
        
    }

    Ok(())
}