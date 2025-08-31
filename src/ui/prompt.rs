use std::error::Error;

use rand::{prelude::StdRng, Rng, SeedableRng};
use tui::{
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  text::Span,
  widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{
  info::{get_hostname, get_tty},
  ui::{prompt_value, util::*, Frame},
  Greeter, Mode, SecretDisplay,
};

use super::common::style::Themed;

pub fn draw(greeter: &mut Greeter, f: &mut Frame) -> Result<(u16, u16), Box<dyn Error>> {
  let theme = &greeter.theme;

  let size = f.size();
  let (x, y, width, height) = get_rect_bounds(greeter, size, 0);

  let container_padding = greeter.container_padding();
  let prompt_padding = greeter.prompt_padding();

  let prompt_container = Rect::new(x, y, width, height);
  let prompt_rect = Rect::new(
    x + container_padding,
    y + container_padding,
    width - (2 * container_padding),
    height - (2 * container_padding),
  );

  let hostname = Span::from(titleize(&fl!(
    "title_authenticate",
    hostname = get_hostname(),
    tty = get_tty()
  )));

  let prompt_block = Block::default()
    .title(hostname)
    .title_style(theme.of(&[Themed::Title]))
    .style(theme.of(&[Themed::Container]))
    .borders(Borders::ALL)
    .border_type(BorderType::Plain)
    .border_style(theme.of(&[Themed::Border]));

  f.render_widget(prompt_block, prompt_container);

  let should_display_answer = greeter.mode == Mode::Password;

  let constraints = [
    Constraint::Length(1),                                                      // Username
    Constraint::Length(if should_display_answer { prompt_padding } else { 0 }), // Prompt padding
    Constraint::Length(if should_display_answer { 1 } else { 0 }),              // Answer
  ];

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(constraints.as_ref())
    .split(prompt_rect);
  let (username_rect, answer_rect) = (chunks[0], chunks[2]);

  let below_prompt = Rect::new(
    greeter.window_padding(),
    prompt_container.bottom() + 1,
    size.width - greeter.window_padding() * 2,
    size.height - prompt_container.bottom() - greeter.window_padding() - 3,
  );

  let greeting = get_greeting(greeter, below_prompt).style(theme.of(&[Themed::Greet]));
  let greeting_width = greeting.line_width().min(120) as u16;

  // Align to middle
  let [_, greeting_rect, _] = Layout::horizontal(vec![
    Constraint::Fill(1),
    Constraint::Length(greeting_width),
    Constraint::Fill(1),
  ])
  .areas(below_prompt);

  f.render_widget(greeting, greeting_rect);

  let above_prompt = Rect::new(
    greeter.window_padding(),
    greeter.window_padding(),
    size.width - greeter.window_padding() * 2,
    prompt_container.y - greeter.window_padding(),
  );

  let date = get_date(greeter).centered();
  let figlet_time = get_figlet_time(greeter).centered();

  // Align just above prompt
  let [_, above_prompt] = Layout::vertical(vec![
    Constraint::Fill(1),
    Constraint::Length(figlet_time.line_count(120) as u16 + 1),
  ])
  .areas(above_prompt);

  let [date_rect, time_rect] = Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)]).areas(above_prompt);

  f.render_widget(date, date_rect);
  f.render_widget(figlet_time, time_rect);

  let username_label = if greeter.user_menu && greeter.username.value.is_empty() {
    let prompt_text = Span::from(fl!("select_user"));

    Paragraph::new(prompt_text).alignment(Alignment::Center)
  } else {
    let username_text = prompt_value(theme, Some(fl!("username")));

    Paragraph::new(username_text)
  };

  let username = greeter.username.get();
  let username_value_text = Span::from(username);
  let username_value = Paragraph::new(username_value_text).style(theme.of(&[Themed::Input]));

  match greeter.mode {
    Mode::Username | Mode::Password | Mode::Action => {
      f.render_widget(username_label, username_rect);

      if !greeter.user_menu || !greeter.username.value.is_empty() {
        f.render_widget(
          username_value,
          Rect::new(
            1 + username_rect.x + fl!("username").chars().count() as u16,
            username_rect.y,
            get_input_width(greeter, width, &Some(fl!("username"))),
            1,
          ),
        );
      }

      let answer_text = if greeter.working {
        Span::from(fl!("wait"))
      } else {
        prompt_value(theme, greeter.prompt.as_ref())
      };

      let answer_label = Paragraph::new(answer_text);

      if greeter.mode == Mode::Password || greeter.previous_mode == Mode::Password {
        f.render_widget(answer_label, answer_rect);

        if !greeter.asking_for_secret || greeter.secret_display.show() {
          let value = match (greeter.asking_for_secret, &greeter.secret_display) {
            (true, SecretDisplay::Character(pool)) => {
              if pool.chars().count() == 1 {
                pool.repeat(greeter.buffer.chars().count())
              } else {
                let mut rng = StdRng::seed_from_u64(0);

                greeter
                  .buffer
                  .chars()
                  .map(|_| pool.chars().nth(rng.gen_range(0..pool.chars().count())).unwrap())
                  .collect()
              }
            }

            _ => greeter.buffer.clone(),
          };

          let answer_value_text = Span::from(value);
          let answer_value = Paragraph::new(answer_value_text).style(theme.of(&[Themed::Input]));

          f.render_widget(
            answer_value,
            Rect::new(
              answer_rect.x + greeter.prompt_width() as u16,
              answer_rect.y,
              get_input_width(greeter, width, &greeter.prompt),
              1,
            ),
          );
        }
      }

      if let (Some(message), message_height) = get_message_height(greeter, container_padding, 1) {
        let message = message.alignment(Alignment::Center);
        let message_rect = Rect::new(x, y + height - 1, width, message_height);
        f.render_widget(message, message_rect);
      }
    }

    _ => {}
  }

  match greeter.mode {
    Mode::Username => {
      let username_length = greeter.username.get().chars().count();
      let offset = get_cursor_offset(greeter, username_length);

      Ok((
        2 + username_rect.x + fl!("username").chars().count() as u16 + offset as u16,
        1 + username_rect.y,
      ))
    }

    Mode::Password => {
      let answer_length = greeter.buffer.chars().count();
      let offset = get_cursor_offset(greeter, answer_length);

      if greeter.asking_for_secret && !greeter.secret_display.show() {
        Ok((
          1 + username_rect.x + greeter.prompt_width() as u16,
          2 + prompt_padding + username_rect.y,
        ))
      } else {
        Ok((
          1 + username_rect.x + greeter.prompt_width() as u16 + offset as u16,
          2 + prompt_padding + username_rect.y,
        ))
      }
    }

    _ => Ok((1, 1)),
  }
}
