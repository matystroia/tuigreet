use std::borrow::Cow;

use ansi_to_tui::IntoText;
use chrono::Local;
use tui::{
  prelude::Rect,
  text::Text,
  widgets::{Paragraph, Wrap},
};

use crate::{fortune::get_figlet, Greeter, Mode};

pub fn titleize(message: &str) -> String {
  format!(" {message} ")
}

pub fn buttonize(message: &str) -> String {
  format!(" {message}")
}

// Determinew whether the cursor should be shown or hidden from the current
// mode and configuration. Usually, we will show the cursor only when expecting
// text entries from the user.
pub fn should_hide_cursor(greeter: &Greeter) -> bool {
  greeter.working
    || greeter.done
    || (greeter.user_menu && greeter.mode == Mode::Username && greeter.username.value.is_empty())
    || (greeter.mode == Mode::Password && greeter.prompt.is_none())
    || greeter.mode == Mode::Users
    || greeter.mode == Mode::Sessions
    || greeter.mode == Mode::Power
    || greeter.mode == Mode::Processing
    || greeter.mode == Mode::Action
}

// Computes the height of the main window where we display content, depending on
// the mode and spacing configuration.
//
// +------------------------+
// |                        | <- container padding
// |        Greeting        | <- greeting height
// |                        | <- auto-padding if greeting
// | Username:              | <- username
// | Password:              | <- password if prompt == Some(_)
// |                        | <- container padding
// +------------------------+
pub fn get_height(greeter: &Greeter) -> u16 {
  let container_padding = greeter.container_padding();
  let prompt_padding = greeter.prompt_padding();

  match greeter.mode {
    Mode::Username | Mode::Action | Mode::Command => (2 * container_padding) + 1,
    Mode::Password => match greeter.prompt {
      Some(_) => (2 * container_padding) + prompt_padding + 2,
      None => (2 * container_padding) + 1,
    },
    Mode::Users | Mode::Sessions | Mode::Power | Mode::Processing => 2 * container_padding,
  }
}

// Get the coordinates and size of the main window area, from the terminal size,
// and the content we need to display.
pub fn get_rect_bounds(greeter: &Greeter, area: Rect, items: usize) -> (u16, u16, u16, u16) {
  let width = greeter.width();
  let height: u16 = get_height(greeter) + items as u16;

  let x = if width < area.width {
    (area.width - width) / 2
  } else {
    0
  };
  let y = if height < area.height {
    (area.height - height) / 2
  } else {
    0
  };

  let (x, width) = if (x + width) >= area.width {
    (0, area.width)
  } else {
    (x, width)
  };
  let (y, height) = if (y + height) >= area.height {
    (0, area.height)
  } else {
    (y, height)
  };

  (x, y, width, height)
}

// Computes the size of a text entry, from the container width and, if
// applicable, the prompt length.
pub fn get_input_width(greeter: &Greeter, width: u16, label: &Option<String>) -> u16 {
  let width = std::cmp::min(greeter.width(), width);

  let label_width = match label {
    None => 0,
    Some(label) => label.chars().count(),
  };

  width - label_width as u16 - 4 - 1
}

pub fn get_cursor_offset(greeter: &mut Greeter, length: usize) -> i16 {
  let mut offset = length as i16 + greeter.cursor_offset;

  if offset < 0 {
    offset = 0;
    greeter.cursor_offset = -(length as i16);
  }

  if offset > length as i16 {
    offset = length as i16;
    greeter.cursor_offset = 0;
  }

  offset
}

pub fn get_greeting(greeter: &Greeter, area: Rect) -> Paragraph {
  let fortune_text = match greeter.fortune.into_text() {
    Ok(text) => text,
    Err(_) => Text::raw(&greeter.fortune),
  };

  let mut paragraph = Paragraph::new(fortune_text).wrap(Wrap { trim: false });

  if paragraph.line_count(120) as u16 > area.height {
    paragraph = Paragraph::new(Text::raw("Fortune too long, unfortunately"));
  }

  paragraph
}

pub fn get_date(greeter: &Greeter) -> Paragraph {
  let date = Local::now()
    .format_localized(&Cow::Owned(fl!("date")), greeter.locale)
    .to_string();

  Paragraph::new(date)
}

pub fn get_figlet_time(greeter: &Greeter) -> Paragraph {
  let time = Local::now().format_localized("%H:%M", greeter.locale).to_string();
  let figlet = get_figlet(&time);

  Paragraph::new(figlet)
}

pub fn get_message_height(greeter: &Greeter, padding: u16, fallback: u16) -> (Option<Paragraph>, u16) {
  if let Some(message) = &greeter.message {
    let width = greeter.width();
    let paragraph = Paragraph::new(message.trim_end()).wrap(Wrap { trim: true });
    let height = paragraph.line_count(width - 4);

    (Some(paragraph), height as u16 + padding)
  } else {
    (None, fallback)
  }
}
