use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputFile,
        InputMessageContent, InputMessageContentText, Me,
    },
    utils::command::BotCommands,
};

use crate::runtimes::build_runtime;
use crate::runtimes::gdrive_runtime::HubWrapper;

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
pub struct ApplicationConfig {
    allowed_users_id: Vec<i64>,
    allowed_chats_id: Vec<i64>,
    #[serde(default, skip_serializing_if = "is_default")]
    private_mode: bool,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

impl ApplicationConfig {
    pub fn set_access_mode(&mut self, access: bool) -> &mut Self {
        self.private_mode = access;
        self
    }

    fn is_id_allowed(&self, id: &i64) -> bool {
        self.allowed_users_id.contains(id) || self.allowed_chats_id.contains(id)
    }
}

fn make_paths_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let paths = build_runtime::get_projects();

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

pub async fn message_handler(
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

pub async fn inline_query_handler(
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

pub async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    gdrive: Arc<HubWrapper>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(project_name) = q.data {
        let text = format!("Project to build: {project_name}");

        bot.answer_callback_query(q.id).await?;

        let message = q.message.unwrap();
        bot.edit_message_text(message.chat.id, message.id, text)
            .await?;

        log::info!("Received build query for: {}", project_name);

        tokio::spawn(
            async move { wait_for_build(bot, message.chat.id, gdrive, project_name).await },
        );
    }

    Ok(())
}

pub async fn wait_for_build(
    bot: Bot,
    id: ChatId,
    gdrive: Arc<HubWrapper>,
    project_name: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match build_runtime::unity_build(&project_name).await {
        Ok(output) => {
            let mut doc = bot.send_document(id, InputFile::file(output.log_path.unwrap()));
            doc.caption = Some("Build complete successfully".to_owned());
            doc.await?;
            match gdrive.upload_file(output.build_path).await {
                Ok(file) => {
                    if let Some(build_download_url) = file.web_content_link {
                        bot.send_message(id, build_download_url).await?;
                    }
                }
                Err(e) => println!("{e}"),
            }
        }
        Err(_) => {
            bot.send_message(id, "Build failed").await?;
        }
    };
    Ok(())
}
