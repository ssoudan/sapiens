use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::{cmark_resume, State};
use sapiens::chains::Message;
use sapiens::context::{ChatEntry, ChatEntryFormatter, MessageFormatter};
use sapiens::models::Role;

/// Chat entry formatter that renders the chat entry in markdown
pub(crate) struct Formatter {}

impl ChatEntryFormatter for Formatter {
    fn format(&self, entry: &ChatEntry) -> String {
        let msg = entry.msg.clone();
        match entry.role {
            Role::User => format!(":earth_americas:\n{msg}"),
            Role::Assistant => format!(":robot:\n{msg}"),
            Role::System => format!(":rooster:\n{msg}"),
            Role::Function => format!(":gear:\n{msg}"),
            Role::Tool => format!(":wrench:\n{msg}"),
        }
    }
}

impl MessageFormatter for Formatter {
    fn format(&self, msg: &Message) -> String {
        msg.to_string()
    }
}

/// Size in characters of an event once rendered in markdown
#[allow(clippy::match_same_arms)]
fn md_event_size(event: &Event) -> usize {
    match event {
        Event::Text(text) => text.len(),
        Event::Code(text) => text.len(),
        Event::Html(text) => text.len(),
        Event::FootnoteReference(text) => text.len(),
        Event::SoftBreak => 1,
        Event::HardBreak => 2,
        Event::Rule => 1,
        Event::TaskListMarker(_) => 3,
        Event::Start(tag) => {
            let len = match tag {
                Tag::Paragraph => 2,
                Tag::Heading {
                    level, id, classes, ..
                } => {
                    *level as usize
                        + 1
                        + id.as_ref().map(|x| x.len()).unwrap_or_default()
                        + classes.iter().map(|x| x.len()).sum::<usize>()
                }
                Tag::BlockQuote => 2,
                Tag::CodeBlock(CodeBlockKind::Indented) => 2,
                Tag::CodeBlock(CodeBlockKind::Fenced(fence)) => fence.len() + 3,
                Tag::List(_) => 2,
                Tag::Item => 2,
                Tag::FootnoteDefinition(d) => d.len() + 3,
                Tag::Table(_) => 4,
                Tag::TableHead => 2,
                Tag::TableRow => 3,
                Tag::TableCell => 3,
                Tag::Emphasis => 2,
                Tag::Strong => 2,
                Tag::Strikethrough => 2,
                Tag::Link {
                    dest_url, title, ..
                } => 4 + dest_url.len() + title.len(),
                Tag::Image {
                    dest_url, title, ..
                } => 4 + dest_url.len() + title.len(),
                Tag::HtmlBlock => 0,
                Tag::MetadataBlock(_) => 0,
            };
            len
        }
        Event::End(_tag) => 4, // random
        Event::InlineHtml(i) => i.len(),
    }
}

/// Sanitize messages for Discord
///
/// Tries to split mardown messages into multiple messages if they are too
/// long for Discord.
///
/// If any split is still too long, truncate it.
pub(crate) fn sanitize_msgs_for_discord(msgs: Vec<String>) -> Vec<String> {
    msgs.into_iter()
        .flat_map(|m| split_msgs(m, 1800))
        .map(|mut x| {
            if x.len() > 1800 - 3 {
                x.truncate(1800);
                x.push_str("...");
            }
            x
        })
        .collect()
}

#[allow(clippy::match_same_arms)]
#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_block_delimiter(t: &TagEnd) -> bool {
    match t {
        TagEnd::Paragraph => true,
        TagEnd::Heading(_) => false,
        TagEnd::BlockQuote => true,
        TagEnd::CodeBlock => true,
        TagEnd::HtmlBlock => true,
        TagEnd::List(_) => true,
        TagEnd::Item => false,
        TagEnd::FootnoteDefinition => false,
        TagEnd::Table => true,
        TagEnd::TableHead => false,
        TagEnd::TableRow => false,
        TagEnd::TableCell => false,
        TagEnd::Emphasis => false,
        TagEnd::Strong => false,
        TagEnd::Strikethrough => false,
        TagEnd::Link => false,
        TagEnd::Image => false,
        TagEnd::MetadataBlock(_) => true,
    }
}

/// Split a message into multiple messages if it is too long
///
/// This is a workaround for the fact that Discord has a 2000 character
/// limit for messages.
/// We want messages to be as long as possible but not longer than 2000.
/// The message is in markdown format.
/// We want to split on sections if possible falling back on newlines
/// outside of code blocks.
fn split_msgs(msg: String, max_size: usize) -> Vec<String> {
    // Simple first
    if msg.len() <= max_size {
        return vec![msg];
    }

    // Markdown split on sections
    let mut buf = String::with_capacity(msg.len() + 128);
    let mut msgs = vec![];

    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);

    let mut current_size = 0;

    let mut state: Option<State> = None;
    for event in Parser::new_ext(&msg, options) {
        let event_size = md_event_size(&event);
        // println!(
        //     "current_size: {}, event_size: {} [{:?}]",
        //     current_size, event_size, event
        // );

        if let Event::End(t) = &event {
            if is_block_delimiter(t) & (current_size + event_size > max_size / 2)
                || (current_size + event_size > max_size * 2 / 3)
            {
                if let Some(state) = state {
                    state.finalize(&mut buf).unwrap();
                    msgs.push(buf.clone());
                    buf.clear();
                }

                current_size = 0;
                state = None;
            }
        } else if current_size + event_size > max_size {
            if let Some(state) = state {
                state.finalize(&mut buf).unwrap();
                msgs.push(buf.clone());
                buf.clear();
            }

            current_size = 0;
            state = None;
        }

        current_size += event_size;

        state = cmark_resume(std::iter::once(event), &mut buf, state.take())
            .unwrap()
            .into();
    }
    if let Some(state) = state {
        state.finalize(&mut buf).unwrap();
    };

    // remove messages that are only whitespace
    msgs.retain(|msg| !msg.trim().is_empty());

    msgs
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use insta::assert_debug_snapshot;
    use pulldown_cmark_to_cmark::cmark;

    use super::*;

    #[test]
    fn estimate_size_in_markdown_one_by_one() {
        let md = indoc! {
              r#"This is a test message that is too long for Discord
       
                # This is a section
               
                This is the second paragraph
                  
                ```python
                print("Hello world")
                ```
                    
                This is the third paragraph
               
                - This is a list
                   - This is another list item
                   - This is another list item
                - This is another list item
               
                ## A subsection
               
                -----------
                | A | B   |
                |---|-----|
                | 1 | 232 |
                | 2 | 3   |
                -----------
               
                   This is the fourth paragraph
                    
                # This is another section
               
                This is the first paragraph of another section                                               
                "#}
        .to_string();

        // normalize
        let mut options = Options::all();
        options.remove(Options::ENABLE_SMART_PUNCTUATION);
        let mut normalized_md = String::with_capacity(md.len() + 128);
        let state = cmark(Parser::new_ext(&md, options), &mut normalized_md);
        if let Ok(state) = state {
            state.finalize(&mut normalized_md).unwrap();
        }

        // assert_display_snapshot!(normalized_md);

        // count on normalized
        let mut options = Options::all();
        options.remove(Options::ENABLE_SMART_PUNCTUATION);
        let mut estimated_normalized_size = 0;
        for event in Parser::new_ext(&normalized_md, options) {
            estimated_normalized_size += md_event_size(&event);
        }

        assert!(
            estimated_normalized_size >= normalized_md.len(),
            "estimated_normalized_size: {}, normalized_md.len(): {}",
            estimated_normalized_size,
            normalized_md.len()
        );
    }

    #[test]
    fn test_split_msgs() {
        let msg = indoc! {
        r#"This is a test message that is too long for Discord
               
                # This is a section
               
                This is the second paragraph
                  
                ```python
                print("Hello world")
                ```
                    
                This is the third paragraph which is much longer than the other.
                    
                ## A subsection
               
                   This is the fourth paragraph
                    
                # This is another section
               
                This is the first paragraph of another section
                              
               "#}
        .to_string();

        let max_size = 100;

        let msgs = split_msgs(msg, max_size);

        assert_debug_snapshot!(msgs);

        // check that all messages are shorter than max_size
        for msg in &msgs {
            assert!(msg.len() <= max_size);
        }
    }
}
