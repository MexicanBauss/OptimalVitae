#![allow(non_snake_case)]
use reqwest::Client;
use serde_json::Value;
use futures_util::StreamExt;
use std::io::{self, stdin, stdout, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    loop {

        let mut s=String::new();
        print!("Please enter some text: ");
        let _=stdout().flush();
        stdin().read_line(&mut s).expect("Did not enter a correct string");

        // Create the JSON body
        let body = serde_json::json!({
            "model": "llama3",
            "prompt": s
        });

        // Convert the JSON body to a string
        let body_string = body.to_string();

        // Send the POST request
        let res = client
            .post("http://localhost:11434/api/generate")
            .header("Content-Type", "application/json")
            .body(body_string)
            .send()
            .await?;

        // Ensure the response is successful
        if res.status().is_success() {
            let mut stream = res.bytes_stream();
            let mut buffer: String = String::new();

            while let Some(item) = stream.next().await {
                match item {
                    Ok(chunk) => {
                        // Decode the chunk as a UTF-8 string
                        let chunk_str: std::borrow::Cow<str> = String::from_utf8_lossy(&chunk);
                        buffer.push_str(&chunk_str);

                        // Try to parse the buffer as JSON
                        if let Ok(json_response) = serde_json::from_str::<Value>(&buffer) {
                            // Clear the buffer after successfully parsing
                            buffer.clear();

                            if let Some(response_text) = json_response.get("response").and_then(|v| v.as_str()) {
                                // Print the response text as is
                                print!("{}", response_text);
                                io::stdout().flush()?;
                            } else {
                                eprintln!("Invalid JSON structure: 'response' field not found or not a string");
                            }

                        }
                    },
                    Err(e) => {
                        eprintln!("Error while streaming: {}", e);
                    }
                }
            }
        } else {
            eprintln!("Request failed with status: {}", res.status());
        }

        println!("");
        
    }
    
    Ok(())
}
