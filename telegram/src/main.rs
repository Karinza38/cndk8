use reqwest::{header::*, Error};
use std::fs::OpenOptions;
use std::io::{self, Write};
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    net::Download,
    prelude::*,
    types::{ChatId, MediaKind, MediaPhoto, MediaText, Message, MessageId, MessageKind, UserId},
    utils::command::BotCommands,
};
use tokio::fs;

extern crate mime;

use scraper::{Html, Selector};

// fn _get_api_key() -> String {
//     match std::env::var("HETZNER_API_KEY") {
//         Ok(key) => key,
//         Err(_) => {
//             panic!("Please set the HETZNER_API_KEY environment variable");
//         }
//     }
// }

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

// this should be an env variable that can be set on runtime.
fn get_brain_location() -> String {
    match std::env::var("BRAIN_LOCATION") {
        Ok(location) => location,
        Err(_) => panic!("Please set the BRAIN_LOCATION environment variable"),
    }
}
// const BRAIN_LOCATION: &str = "/Users/foursixnine/Library/Mobile Documents/iCloud~md~obsidian/Documents/codex.foursixnine.io/Codex/00-Captured.md";

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
}

// commands

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    // #[command(description = "handle a username.")]
    // Username(String),
    // #[command(description = "handle a username and an age.", parse_with = "split")]
    // UsernameAndAge { username: String, age: u8 },
    #[command(description = "displaly Help this text.")]
    Help,
    #[command(description = "start stufftuff")]
    Start,
    #[command(description = "cancel stuff")]
    Cancel,
}

#[derive(Clone)]
struct ConfigParameters {
    bot_maintainer: UserId,
}

const PARAMETERS: ConfigParameters = ConfigParameters {
    bot_maintainer: UserId(51739298), // Paste your ID to run this bot.
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, update_handler())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn update_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(start))
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::Start].endpoint(handle_message));

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    let help_message = format!(
        "help has been invoked, your user id is {}",
        msg.from().expect("User should have id").id
    );
    bot.send_message(msg.chat.id, help_message).await?;
    Ok(())
}

async fn cancel(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "cancel has been invoked")
        .await?;
    Ok(())
}

async fn start(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "This is the start").await?;
    Ok(())
}

async fn error_handler(error: Error) {
    todo!();
}

async fn handle_message(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let hm = bot
        .send_message(msg.chat.id.clone(), "This is the handle_message")
        .reply_to_message_id(msg.id)
        .await?;

    let _is_owner = |msg: &Message| {
        msg.from()
            .map(|user| {
                // let reply = format!("Your id is {}", user.id);
                user.id == PARAMETERS.bot_maintainer
            })
            .unwrap_or_default()
    };

    if !_is_owner(&msg) {
        return Ok(());
    }

    match msg.kind {
        MessageKind::Common(chat) => {
            match chat.media_kind {
                MediaKind::Text(content) => {
                    handle_text_content(bot.clone(), msg.chat.id, msg.id, Some(content)).await?;
                }
                MediaKind::Photo(content) => {
                    bot.send_message(msg.chat.id, "Got Photo!")
                        .reply_to_message_id(msg.id)
                        .await?;
                    handle_photo_content(bot.clone(), msg.chat.id, msg.id, Some(content.clone()))
                        .await?;
                    //log::debug!("{:#?}", content);
                    // to download the photo
                }
                _ => {
                    bot.send_message(msg.chat.id, "Media::Kind Type not implemented")
                        .reply_to_message_id(msg.id)
                        .await?;
                    log::debug!("{:#?} not implemented", chat);
                    // log::debug!("{:#?} not implemented", msg);
                } //todo!(), // todo for media_kind
            }
        }
        // _ => todo!(), //todo for msg kind
        _ => {
            bot.send_message(msg.chat.id, "MessageKind not implemented")
                .reply_to_message_id(msg.id)
                .await?;
        } //todo!(), // todo for media_kind
    };
    // }
    let msg_id = hm.id;
    log::debug!("message id: {:#?}", &msg_id);
    bot.delete_message(msg.chat.id, msg_id).await?;
    Ok(())
}

async fn handle_photo_content(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    message_photo: Option<MediaPhoto>,
) -> HandlerResult {
    bot.send_message(chat_id, "Got Photo!")
        .reply_to_message_id(message_id)
        .await?;
    let content = message_photo.unwrap();
    log::info!("photo: {:#?}", content);
    // log::debug!("object: {:#?}", content);
    let photo = content.photo.last().unwrap();
    let file = bot.get_file(photo.file.id.clone()).await?;
    let file_id = file.path.clone();
    let file_name = file_id.split('/').last().unwrap();
    let file_path = format!("./tmp/tg/{}", file_name);
    log::debug!("file_path: {}", file_path);
    fs::create_dir_all("./tmp/tg/").await?;
    let mut file = fs::File::create(file_path.clone()).await?;
    log::debug!("Debugging file:\n{:#?}", file);
    bot.download_file(&file_id, &mut file).await?;
    let markdown = format!("- ![{}]({})\n", &file_path, &file_path);
    // let markdown = format!(format, text_part, text_url, entity.kind);
    log::debug!("will insert:");
    log::debug!("{}", markdown);
    // log::info!("object: {:#?}", full_url);
    match append_to_brain(&markdown, SecondBrainSupportedFormats::Markdown) {
        Ok(()) => {
            bot.send_message(chat_id, "Saved photo!")
                .reply_to_message_id(message_id)
                .await?
        }
        _ => panic!("{:?}", &markdown),
    };
    Ok(())
}

async fn handle_text_content(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    message_text: Option<MediaText>,
) -> HandlerResult {
    bot.send_message(chat_id, "Got text message")
        .reply_to_message_id(message_id)
        .await?;

    let content = message_text.unwrap();
    log::info!("text: {}", content.text);

    // TODO: thread 'tokio-runtime-worker' panicked at 'failed trying to parse >: https://thght.works/3vZX6<: RelativeUrlWithoutBase', telegram/src/main.rs:219:40
    let entity = content.entities.first().unwrap();
    let mut markdown = String::new();
    markdown = match entity {
        MediaText => {
            let md = process_media_text(content.clone()).await;
            // need to iterate over the entities in MediaText.entities, and properly format the markdown
            log::debug!(
                "TODO: needs to be implemented for mediatext, so media->text->parse entities"
            );
            md
        }
    };
    log::debug!("plain text is: {}", markdown);
    log::warn!("{:#?} not implemented", content);
    log::debug!("found {}", content.entities.len());
    //    !todo!("Proper implementation for MediaText is still missing");
    append_to_brain(&markdown, SecondBrainSupportedFormats::Markdown)?;
    Ok(())
}

enum SecondBrainSupportedFormats {
    Markdown,
}

use teloxide::types::MessageEntityKind::*;
use teloxide::types::*;

async fn process_media_text(text: MediaText) -> String {
    let mut markdown = String::new();
    markdown.push_str(&text.text);
    for entity in text.entities.iter() {
        match entity.kind {
            MessageEntityKind::Bold => {
                log::debug!("bold: : {:#?}", text.text);
            }
            MessageEntityKind::Italic => {
                log::debug!("italic: : {:#?}", text.text);
            }
            MessageEntityKind::Url => {
                log::debug!("url: : {:#?}", text.clone());
                log::debug!("url: : {:#?},\n{:#?}", markdown.clone(), entity);
                // let text_url = &text.text[entity.offset..entity.length];
                // let text_part = &text.text[0..entity.offset];
                let title_url = match get_website_title(text.text.as_str()).await {
                    Ok(title) => title.to_string(),
                    Err(e) => {
                        log::debug!("{:?}\n error invoked from {}", e, line!());
                        "error in url".to_string()
                    }
                };
                markdown.push_str(
                    format!("\n    -[{}]({})\n", title_url.trim(), text.text.as_str()).as_str(),
                );
                // let markdown = format!(format, text_part, text_url, entity.kind);
                log::debug!("will insert:");
                log::debug!("{}", markdown);
            }
            _ => {
                log::debug!("generic : {:#?}, {:#?}", text.text, entity.kind)
            }
        }
        let format = match entity.kind {
            MessageEntityKind::Bold => "bold",
            MessageEntityKind::Italic => "italic",
            MessageEntityKind::Url => "link",
            _ => "plain",
        };
        markdown.push_str(&format!("{}: [{}]({})\n", format, "foo", "bar"));
    }
    markdown
}

// TODO: needs refactoring.
// TODO: ultimately, this will be part of SecondBrainManager api, so becomes only a call with just
// markdown text to add.
fn append_to_brain(text: &str, format: SecondBrainSupportedFormats) -> io::Result<()> {
    let brain_location = get_brain_location();
    let mut file = OpenOptions::new()
        .append(true)
        .open(brain_location)
        .unwrap();
    file.write_all(text.as_bytes())
        .expect("failed to write/append to brain");
    Ok(())
}

async fn get_website_title(url: &str) -> Result<String, reqwest::Error> {
    let this_url = match reqwest::Url::parse(url) {
        Ok(result) => result,
        Err(e) => panic!("Can't recover '{:#?}'\nurl:{:#?}", url, e),
    };

    let title;
    let host = this_url.host_str();
    match host {
        Some("onlyfans.com") => {
            title = format!(
                "OF of {}",
                this_url
                    .path_segments()
                    .expect("broken")
                    .collect::<Vec<_>>()[0]
            )
        }
        Some("fansly.com") => {
            title = format!(
                "FSL of {}",
                this_url
                    .path_segments()
                    .expect("broken")
                    .collect::<Vec<_>>()[0]
            )
        }
        Some("instagram.com") => {
            title = format!(
                "OF of {}",
                this_url
                    .path_segments()
                    .expect("broken")
                    .collect::<Vec<_>>()[0]
            )
        }
        Some("twitter.com") | Some("x.com") => {
            title = format!(
                "Tweet from {}",
                this_url
                    .path_segments()
                    .expect("broken")
                    .collect::<Vec<_>>()[0]
            )
        }
        Some("facebook.com") => {
            title = format!(
                "Facebook link {}",
                this_url
                    .path_segments()
                    .expect("broken")
                    .collect::<Vec<_>>()[0]
            )
        }
        _ => {
            // // Convert the response body to a string
            // let body_str = String::from_utf8(body.to_vec())?;
            // Parse the title from the HTML
            // Send GET request to the specified URL
            let response = reqwest::get(url).await?;

            // response.headers();

            // Read the response body as bytes
            let body_str = response.text().await?;
            title = parse_website_title(&body_str);
        }
    }

    Ok(title)
}

// trait From<T> :Sized {
//     fn from(&self) -> Self;
// }

fn mimetype_has_title(_content_type: HeaderValue) -> bool {
    // let mime_type: Mime = content_type;

    // match mime_type {
    //     Some(mime::TEXT_HTML) => true,
    //     _ => false,
    //     None => panic!("No type found for: {:#?}", content_type),
    // }
    todo!("mimetype_has_title Not implemented");
}

fn parse_website_title(html: &str) -> String {
    // let document = Html::parse_document(&html);
    // let selector = Selector::parse("title");
    // Extract the title from the HTML using simple string manipulation
    let document = Html::parse_document(html);
    let title_selector = Selector::parse("title").unwrap();
    let title_text = document
        .select(&title_selector)
        .next()
        .map(|x| x.inner_html());

    let binding = title_text.expect("No title found in HTML document");
    let title_string = binding.trim();
    log::debug!(
        "Looks like a title was found, but site had to be parsed: {}",
        line!()
    );
    title_string.to_string()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    #[should_panic]
    pub async fn test_get_website_title_invalid_url() {
        let _url = "g https://twitter.com/aakashg0/status/1666962728055889920?s=52&t=scqAqSz4d-mfoKQNQq-fv";
        let the_response = get_website_title(_url).await.unwrap();
        let expected = "Broken link";
        assert_eq!(expected, the_response);
    }

    #[tokio::test]
    async fn test_media_text() {
        let media_text = MediaText {
            text: "https://www.theregister.com/2024/12/30/att_verizon_confirm_salt_typhoon_breach/"
                .to_string(),
            entities: vec![MessageEntity {
                kind: MessageEntityKind::Url,
                offset: 0,
                length: 79,
            }],
        };

        assert_eq!(
            media_text.text,
            "https://www.theregister.com/2024/12/30/att_verizon_confirm_salt_typhoon_breach/"
        );
        assert_eq!(media_text.entities.len(), 1);
        assert_eq!(media_text.entities[0].kind, MessageEntityKind::Url);
        assert_eq!(media_text.entities[0].offset, 0);
        assert_eq!(media_text.entities[0].length, 79);
    }

    #[tokio::test]
    async fn test_media_text_with_multiple_entities() {
        let media_text = MediaText {
            text: "Check this out: https://example.com and https://anotherexample.com".to_string(),
            entities: vec![
                MessageEntity {
                    kind: MessageEntityKind::Url,
                    offset: 16,
                    length: 19,
                },
                MessageEntity {
                    kind: MessageEntityKind::Url,
                    offset: 40,
                    length: 22,
                },
            ],
        };

        assert_eq!(media_text.entities.len(), 2);
        assert_eq!(media_text.entities[0].kind, MessageEntityKind::Url);
        assert_eq!(media_text.entities[0].offset, 16);
        assert_eq!(media_text.entities[0].length, 19);
        assert_eq!(media_text.entities[1].kind, MessageEntityKind::Url);
        assert_eq!(media_text.entities[1].offset, 40);
        assert_eq!(media_text.entities[1].length, 22);

        let simple_markdown = process_media_text(media_text);
        assert_eq!(
            simple_markdown.await,
            "Check this out: https://example.com and https://anotherexample.com"
        );
    }

    #[tokio::test]
    async fn test_media_text_with_many_entities() {
        let media_text = MediaText {
    text: "Five whys (or 5 whys) is an iterative interrogative technique used to explore the cause-and-effect relationships underlying a particular problem.[1] The primary goal of the technique is to determine the root cause of a defect or problem by repeating the question \"why?\" five times, each time directing the current \"why\" to the answer of the previous \"why\". The method asserts that the answer to the fifth \"why\" asked in this manner should reveal the root cause of the problem.[2]".to_string(),
    entities: [
        MessageEntity {
            kind: Bold,
            offset: 28,
            length: 33,
        },
        MessageEntity {
            kind: TextLink {
                url: reqwest::Url::parse("https://helpfulprofessor.com/cause-and-effect-examples/").unwrap(),
                    },
            offset: 82,
            length: 16,
        },
        MessageEntity {
            kind: TextLink {
                url: reqwest::Url::parse("https://en.wikipedia.org/wiki/Five_whys").unwrap(),
            },
            offset: 315,
            length: 3,
        },
        MessageEntity {
            kind: Italic,
            offset: 406,
            length: 3,
        },
    ].to_vec()
    };
        assert_eq!(
            media_text.text,
            "Five whys (or 5 whys) is an iterative interrogative technique used to explore the cause-and-effect relationships underlying a particular problem.[1] The primary goal of the technique is to determine the root cause of a defect or problem by repeating the question \"why?\" five times, each time directing the current \"why\" to the answer of the previous \"why\". The method asserts that the answer to the fifth \"why\" asked in this manner should reveal the root cause of the problem.[2]"
        );
        assert_eq!(media_text.entities.len(), 4);
        let markdown = process_media_text(media_text).await;
        assert_eq!(markdown, "Hello world");
    }
    // #[test]
    // fn test_process_header_mimetype() {
    //     // let mut map = HeaderMap::new();
    //     let current_mime = HeaderValue::from_static(&mime::TEXT_HTML.to_string());
    //     // map.append(CONTENT_TYPE, current_mime);
    //     // assert_eq!(map.get(CONTENT_TYPE).unwrap(), mime::TEXT_HTML.to_string())
    //     let result = mimetype_has_title(current_mime);
    //     //assert!(result, true);
    // }

    //
    // * “One good test is worth a thousand expert opinions.” \n
    // * – Wernher von Braun @ twitter https://test.com
    // *
    // * thght.works/3ghJZ9t => problem
    // thread 'tokio-runtime-worker' panicked at 'failed trying to parse >: https://thght.works/3vZX6<: RelativeUrlWithoutBase', telegram/src/main.rs:219:40
    //
    // // This has to be converted to a json object
    // DEBUG telegram                          > object: MediaText {
    //    text: "Santiago Zarate, [Jul 8, 2023 at 20:32]\nhttps://www.reddit.com/user/Remarkable-Goat-973/",
    //    entities: [
    //        MessageEntity {
    //            kind: MessageEntityKind::Url,
    //            offset: 40,
    //            length: 48,
    //        },
    //    ],
    //}
    // * */
}
