use clap::{Parser, Subcommand};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::{Command, ExitCode},
};

const CONFIG_DIR: &str = "/usr/arc-ai";
const KEY_FILE: &str = "/usr/arc-ai/groq.key";
const GROQ_URL: &str = "https://api.groq.com/openai/v1/chat/completions";

#[derive(Parser)]
#[command(name = "arc-ai", version, about = "Natural language Linux command runner")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Natural language command text
    text: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Store Groq API key
    Set {
        /// Groq API key
        key: String,
    },
}

#[derive(Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct GroqResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Set { key }) => {
            if let Err(e) = save_key(&key) {
                eprintln!("ERROR: failed to save Groq key: {e}");
                return ExitCode::FAILURE;
            }

            println!("Groq key saved.");
            ExitCode::SUCCESS
        }

        None => {
            let Some(text) = cli.text else {
                eprintln!("usage: arc-ai \"command text\"");
                eprintln!("       arc-ai set [Your-groq-key]");
                return ExitCode::FAILURE;
            };

            let key = match load_key() {
                Ok(k) => k,
                Err(_) => {
                    eprintln!("ERROR: Groq API key not configured.");
                    eprintln!("Run: arc-ai set [Your-groq-key]");
                    return ExitCode::FAILURE;
                }
            };

            let generated = match generate_command(&key, &text).await {
                Ok(cmd) => cmd,
                Err(e) => {
                    eprintln!("ERROR: failed to generate command: {e}");
                    return ExitCode::FAILURE;
                }
            };

            if is_dangerous(&generated) {
                eprintln!("ERROR: refusing dangerous command:");
                eprintln!("{generated}");
                return ExitCode::FAILURE;
            }

            println!("Generated command:");
            println!("{generated}");
            print!("Execute? [y/N]: ");
            io::stdout().flush().ok();

            let mut answer = String::new();
            io::stdin().read_line(&mut answer).ok();

            if answer.trim().to_lowercase() != "y" {
                eprintln!("Cancelled.");
                return ExitCode::FAILURE;
            }

            let status = Command::new("sh")
                .arg("-c")
                .arg(&generated)
                .status();

            match status {
                Ok(s) if s.success() => ExitCode::SUCCESS,
                Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
                Err(e) => {
                    eprintln!("ERROR: failed to execute command: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

fn save_key(key: &str) -> io::Result<()> {
    fs::create_dir_all(CONFIG_DIR)?;
    fs::write(KEY_FILE, key.trim())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(KEY_FILE, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

fn load_key() -> io::Result<String> {
    let key = fs::read_to_string(Path::new(KEY_FILE))?;
    Ok(key.trim().to_string())
}

async fn generate_command(key: &str, user_text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let system_prompt = r#"
You are Arc, a Linux shell command generator.

Rules:
- Output ONLY one shell command.
- No markdown.
- No explanations.
- No code fences.
- No comments.
- Prefer safe commands.
- Use the current working directory unless user says otherwise.
- Do not use sudo unless explicitly requested.
- Do not delete files unless explicitly requested.
"#;

    let body = GroqRequest {
        model: "llama-3.3-70b-versatile".to_string(),
        temperature: 0.0,
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_text.to_string(),
            },
        ],
    };

    let client = reqwest::Client::new();

    let res = client
        .post(GROQ_URL)
        .header(AUTHORIZATION, format!("Bearer {key}"))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Groq API error {status}: {text}").into());
    }

    let parsed: GroqResponse = res.json().await?;
    let cmd = parsed
        .choices
        .first()
        .ok_or("missing Groq response choice")?
        .message
        .content
        .trim()
        .to_string();

    Ok(cmd)
}

fn is_dangerous(cmd: &str) -> bool {
    let lowered = cmd.to_lowercase();

    let blocked = [
        "rm -rf /",
        "rm -rf /*",
        "mkfs",
        "dd if=",
        ":(){",
        "chmod -r 777 /",
        "chown -r",
        "> /dev/sda",
        "shutdown",
        "reboot",
        "poweroff",
    ];

    blocked.iter().any(|bad| lowered.contains(bad))
}