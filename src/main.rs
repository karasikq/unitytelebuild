use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, Me,
    },
    utils::command::BotCommands,
};
use unitytelebuild::unitybuild::process::{BuildPlatform, LogBehaviour, UnityProcess};

#[derive(BotCommands)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum BotCommand {
    #[command(description = "Help")]
    Help,
    #[command(description = "Start")]
    Start,
    #[command(description = "Select project to build")]
    Build,
}

#[derive(Serialize, Deserialize)]
struct ApplicationConfig {
    allowed_users_id: Vec<i64>,
    allowed_chats_id: Vec<i64>,
    #[serde(default, skip_serializing_if = "is_default")]
    private_mode: bool,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

impl ApplicationConfig {
    fn is_id_allowed(&self, id: &i64) -> bool {
        self.allowed_users_id.contains(id) || self.allowed_chats_id.contains(id)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting unitytelebuild bot...");

    let config_file = fs::File::open("config.json").expect("Failed to load 'config.json'");
    let mut config: ApplicationConfig =
        serde_json::from_reader(config_file).expect("Failed to parse 'config.json'");
    config.private_mode = dotenv::var("PRIVATE_MODE")
        .expect("Environment variable PRIVATE_MODE should be set in '.env'")
        .parse::<bool>()
        .unwrap();

    let bot = Bot::new(dotenv::var("TELOXIDE_BOT_TOKEN").unwrap());

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler))
        .branch(Update::filter_inline_query().endpoint(inline_query_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![Arc::new(config)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

fn get_projects() -> Vec<PathBuf> {
    fs::read_dir(
        dotenv::var("PROJECTS_LOCATION")
            .expect("Environment variable PROJECTS_LOCATION should be set in '.env'"),
    )
    .expect("Cannot read PROJECTS_LOCATION directory")
    .filter(|f| f.as_ref().unwrap().path().is_dir())
    .map(|f| f.unwrap().path())
    .collect::<Vec<_>>()
}

fn unity_build() {
    let paths = fs::read_dir(
        dotenv::var("PROJECTS_LOCATION")
            .expect("Environment variable PROJECTS_LOCATION should be set in '.env'"),
    )
    .expect("Cannot read PROJECTS_LOCATION directory")
    .filter(|f| f.as_ref().unwrap().path().is_dir())
    .map(|f| f.unwrap().path())
    .collect::<Vec<_>>();

    let project_path = paths.last().unwrap().to_str().unwrap();
    println!("{:?}", project_path);

    let default_log_path = Path::new(project_path)
        .join(
            dotenv::var("UNITY_LOG_PATH")
                .expect("Environment variable UNITY_LOG_PATH should be set in '.env'"),
        )
        .join("androind_build.log");

    let log_to_stdout = dotenv::var("LOG_TO_STDOUT")
        .expect("Environment variable LOG_TO_STDOUT should be set in '.env'")
        .parse::<bool>()
        .unwrap();

    let mut unity_process = UnityProcess::new();
    let process = unity_process.set_bin(dotenv::var("UNITY_BIN").unwrap().into());
    if log_to_stdout {
        process.set_log_behavior(LogBehaviour::StdoutFile);
    } else {
        process
            .set_log_behavior(LogBehaviour::File)
            .set_log_path(default_log_path.to_str().unwrap().into());
    }
    process
        .set_platform(BuildPlatform::AndroidDevelopment)
        .set_project_path(project_path.into());

    let output = process.build().unwrap_or_else(|_| {
        panic!(
            "Failed to execute Unity process. See logs at {0}",
            default_log_path.to_str().unwrap()
        )
    });

    println!("Process status: {}", output.status);
    /* println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr)); */
}

fn make_paths_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let paths = get_projects();

    for path_chunk in paths.chunks(3) {
        let row = path_chunk
            .iter()
            .map(|path| {
                InlineKeyboardButton::callback(
                    path.file_name().unwrap().to_string_lossy(),
                    path.file_name().unwrap().to_string_lossy(),
                )
            })
            .collect();

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

async fn message_handler(
    bot: Bot,
    config: Arc<ApplicationConfig>,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    log::info!("Received message from {0}", msg.chat.id);

    let allow_proccess = match config.private_mode {
        true => config.is_id_allowed(&msg.chat.id.0),
        false => true,
    };

    if allow_proccess {
        if let Some(text) = msg.text() {
            match BotCommands::parse(text, me.username()) {
                Ok(BotCommand::Help) => {
                    bot.send_message(msg.chat.id, BotCommand::descriptions().to_string())
                        .await?;
                }
                Ok(BotCommand::Start) => {
                    bot.send_message(msg.chat.id, "Type /build for list projects")
                        .await?;
                }
                Ok(BotCommand::Build) => {
                    let keyboard = make_paths_keyboard();
                    bot.send_message(msg.chat.id, "Select project: ")
                        .reply_markup(keyboard)
                        .await?;
                }
                Err(_) => {
                    bot.send_message(msg.chat.id, "Command not found!").await?;
                }
            }
        }
    } else {
        log::info!("Ignore message from {0}", msg.chat.id);
    }

    Ok(())
}

async fn inline_query_handler(
    bot: Bot,
    q: InlineQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let choose_project = InlineQueryResultArticle::new(
        "0",
        "Choose project",
        InputMessageContent::Text(InputMessageContentText::new("Select project: ")),
    )
    .reply_markup(make_paths_keyboard());

    bot.answer_inline_query(q.id, vec![choose_project.into()])
        .await?;

    Ok(())
}

async fn callback_handler(bot: Bot, q: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(project_name) = q.data {
        let text = format!("Project to build: {project_name}");

        bot.answer_callback_query(q.id).await?;

        if let Some(Message { id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }

        log::info!("Received build query for: {}", project_name);
    }

    Ok(())
}
