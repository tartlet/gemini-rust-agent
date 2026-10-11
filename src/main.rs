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

fn ask_agent(&mut input: &String) -> &str {
    print!("Type something ... \n");

    use std::io::{self, Write};
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;

    let input = input.trim();
    return input;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY")?;

    let client = Client::new();

    let mut previous_interaction_id: Option<String> = None;
    loop {

        let mut input = String::new();
        input = ask_agent(&input);

        if input == "quit" {
            break;
        }

        let mut request_body = json!({
            "model": "gemini-3.7-flash",
            "input": &input
        });

        // look inside previous_interaction_id. If it contains a value, give me that value and call it id
        if let Some(id) = &previous_interaction_id {
            request_body["previous_interaction_id"] = json!(id);
        }

        let response = client
            .post("https://generativelanguage.googleapis.com/v1beta/interactions")
            .header("x-goog-api-key", &api_key)
            .json(&request_body)
            .send()
            .await?;

        let response = response.error_for_status()?;
        let body: InteractionResponse = response.json().await?;
        
        previous_interaction_id = Some(body.id.clone());

        if let Some(text) = extract_text(&body) {
            println!("Gemini: {text}");
        }
    }

    Ok(())
}