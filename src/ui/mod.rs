mod command;
pub mod common;
mod i18n;
pub mod power;
mod processing;
mod prompt;
pub mod sessions;
pub mod users;
mod util;

use std::{
  error::Error,
  io::{self, Write},
  sync::Arc,
};

use sessions::SessionSource;
use tokio::sync::RwLock;
use tui::{
  layout::{Alignment, Constraint, Direction, Layout},
  style::Modifier,
  text::{Line, Span},
  widgets::Paragraph,
  Frame as CrosstermFrame, Terminal,
};
use util::buttonize;

use crate::{info::capslock_status, ui::util::should_hide_cursor, Greeter, Mode};

use self::common::style::{Theme, Themed};
pub use self::i18n::MESSAGES;

const STATUSBAR_INDEX: usize = 2;
const STATUSBAR_LEFT_INDEX: usize = 1;
const STATUSBAR_RIGHT_INDEX: usize = 2;

pub(super) type Frame<'a> = CrosstermFrame<'a>;

enum Button {
  Command,
  Session,
  Power,
  Other,
}

pub async fn draw<B>(greeter: Arc<RwLock<Greeter>>, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>>
where
  B: tui::backend::Backend,
{
  let mut greeter = greeter.write().await;
  let hide_cursor = should_hide_cursor(&greeter);

  if greeter.clear_request {
    terminal.clear().ok();
    greeter.clear_request = false;
  }

  terminal.draw(|f| {
    let theme = &greeter.theme;

    let size = f.size();
    let chunks = Layout::default()
      .constraints(
        [
          Constraint::Length(greeter.window_padding()), // Top vertical padding
          Constraint::Min(1),                           // Main area
          Constraint::Length(1),                        // Status line
          Constraint::Length(greeter.window_padding()), // Bottom vertical padding
        ]
        .as_ref(),
      )
      .split(size);

    let session_source_label = match greeter.session_source {
      SessionSource::Session(_) => fl!("status_session"),
      _ => fl!("status_command"),
    };
    let session_source = greeter.session_source.label(&greeter).unwrap_or("-");

    let status_block_size_right = session_source_label.chars().count() as u16
      + 1
      + session_source.chars().count() as u16
      + 1
      + fl!("status_caps").chars().count() as u16
      + greeter.window_padding();
    let status_block_size_left = (size.width - greeter.window_padding()) - status_block_size_right;

    let status_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
        [
          Constraint::Length(greeter.window_padding()),
          Constraint::Length(status_block_size_left),
          Constraint::Length(status_block_size_right),
          Constraint::Length(greeter.window_padding()),
        ]
        .as_ref(),
      )
      .split(chunks[STATUSBAR_INDEX]);

    let status_left_text = Line::from(vec![
      status_label(theme, format!("F{}", greeter.kb_command)),
      status_value(&greeter, theme, Button::Command, fl!("action_command")),
      Span::from(" "),
      status_label(theme, format!("F{}", greeter.kb_sessions)),
      status_value(&greeter, theme, Button::Session, fl!("action_session")),
      Span::from(" "),
      status_label(theme, format!("F{}", greeter.kb_fortune)),
      status_value(&greeter, theme, Button::Command, "Fortune"),
      Span::from(" "),
      status_label(theme, format!("F{}", greeter.kb_power)),
      status_value(&greeter, theme, Button::Power, fl!("action_power")),
    ]);
    let status_left = Paragraph::new(status_left_text);

    f.render_widget(status_left, status_chunks[STATUSBAR_LEFT_INDEX]);

    let status_right_text = Line::from(
      vec![
        status_label(theme, session_source_label),
        status_value(&greeter, theme, Button::Other, session_source),
        Span::from(" "),
      ]
      .into_iter()
      .chain(capslock_status().then(|| status_label(theme, fl!("status_caps"))))
      .collect::<Vec<_>>(),
    );
    let status_right = Paragraph::new(status_right_text).alignment(Alignment::Right);

    f.render_widget(status_right, status_chunks[STATUSBAR_RIGHT_INDEX]);

    let cursor = match greeter.mode {
      Mode::Command => self::command::draw(&mut greeter, f).ok(),
      Mode::Sessions => greeter.sessions.draw(&greeter, f).ok(),
      Mode::Power => greeter.powers.draw(&greeter, f).ok(),
      Mode::Users => greeter.users.draw(&greeter, f).ok(),
      Mode::Processing => self::processing::draw(&mut greeter, f).ok(),
      _ => self::prompt::draw(&mut greeter, f).ok(),
    };

    if !hide_cursor {
      if let Some(cursor) = cursor {
        f.set_cursor(cursor.0 - 1, cursor.1 - 1);
      }
    }
  })?;

  io::stdout().flush()?;

  Ok(())
}

fn status_label<'s, S>(theme: &Theme, text: S) -> Span<'s>
where
  S: Into<String>,
{
  Span::styled(
    text.into(),
    theme.of(&[Themed::ActionButton]).add_modifier(Modifier::REVERSED),
  )
}

fn status_value<'s, S>(greeter: &Greeter, theme: &Theme, button: Button, text: S) -> Span<'s>
where
  S: Into<String>,
{
  let relevant_mode = match button {
    Button::Command => Mode::Command,
    Button::Session => Mode::Sessions,
    Button::Power => Mode::Power,

    _ => {
      return Span::from(buttonize(&text.into())).style(theme.of(&[Themed::Action]));
    }
  };

  let style = match greeter.mode == relevant_mode {
    true => theme.of(&[Themed::ActionButton]).add_modifier(Modifier::REVERSED),
    false => theme.of(&[Themed::Action]),
  };

  Span::from(buttonize(&text.into())).style(style)
}

fn prompt_value<'s, S>(theme: &Theme, text: Option<S>) -> Span<'s>
where
  S: Into<String>,
{
  match text {
    Some(text) => Span::styled(text.into(), theme.of(&[Themed::Prompt]).add_modifier(Modifier::BOLD)),
    None => Span::from(""),
  }
}
