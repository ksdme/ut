use clap::{CommandFactory, Parser};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
};
use regex::Regex;
use std::io::{self, Stdout};
use tui_textarea::{Input, TextArea};

use crate::tool::Tool;

#[derive(Parser, Debug)]
#[command(name = "regex")]
pub struct RegexTool {}

impl Tool for RegexTool {
    fn cli() -> clap::Command {
        RegexTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<crate::tool::Output>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = App::default();
        let res = run_app_loop(&mut terminal, &mut app);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{err:?}");
        }

        Ok(None)
    }
}

enum InputFocus {
    Regex,
    Sample,
}

struct App<'a> {
    input_focus: InputFocus,

    regex_textarea: TextArea<'a>,
    sample_textarea: TextArea<'a>,

    compiled_regex: Option<Regex>,
    regex_error: Option<String>,
}

impl<'a> Default for App<'a> {
    fn default() -> App<'a> {
        let mut regex_textarea = TextArea::default();
        regex_textarea.set_cursor_line_style(Style::new());

        let mut sample_textarea = TextArea::default();
        sample_textarea.set_cursor_line_style(Style::new());

        App {
            input_focus: InputFocus::Sample,

            regex_textarea,
            sample_textarea,

            compiled_regex: None,
            regex_error: None,
        }
    }
}

impl<'a> App<'a> {
    fn get_sample_text(&self) -> String {
        self.sample_textarea.lines().join("\n")
    }

    fn compile_regex(&mut self) {
        self.compiled_regex = None;
        self.regex_error = None;

        let Some(regex_input) = self.regex_textarea.lines().first() else {
            return;
        };

        if regex_input.is_empty() {
            return;
        }

        match Regex::new(&regex_input) {
            Ok(regex) => {
                self.compiled_regex = Some(regex);
                self.regex_error = None;
            }
            Err(e) => {
                self.compiled_regex = None;
                self.regex_error = Some(format!("Regex error: {}", e));
            }
        }
    }

    fn get_highlighted_text(&self) -> Text<'static> {
        let sample_text = self.get_sample_text();
        let Some(regex) = &self.compiled_regex else {
            return Text::from(sample_text);
        };

        let highlight_styles = &[
            Style::new().bg(Color::LightBlue).fg(Color::Black),
            Style::new().bg(Color::LightGreen).fg(Color::Black),
            Style::new().bg(Color::LightRed).fg(Color::Black),
            Style::new().bg(Color::LightYellow).fg(Color::Black),
            Style::new().bg(Color::Blue).fg(Color::Black),
            Style::new().bg(Color::Green).fg(Color::Black),
            Style::new().bg(Color::Red).fg(Color::White),
            Style::new().bg(Color::Yellow).fg(Color::Black),
            Style::new().bg(Color::Magenta).fg(Color::White),
        ];

        let mut highlights: Vec<(usize, usize, Style)> = vec![];
        for capture in regex.captures_iter(&sample_text) {
            for (group, submatch) in capture.iter().enumerate() {
                if let Some(submatch) = submatch {
                    highlights.push((
                        submatch.start(),
                        submatch.end(),
                        highlight_styles[group % highlight_styles.len()],
                    ));
                }
            }
        }

        // This is a fallback style when a span has no highlight match. Although,
        // to make sure full matches from not being highlighted, we need to make sure
        // this fallback is the last element, even after the sort later.
        highlights.push((0, sample_text.len(), Style::new()));

        // Sort the highlights by their size and start position. This lets us
        // to exit as soon as one overlapping match is found below.
        highlights.sort_by(|a, b| ((a.1 - a.0), a.0).cmp(&((b.1 - b.0, b.0))));

        // All the boundary points in the vector.
        let mut boundaries = highlights
            .iter()
            .flat_map(|(s, e, _)| vec![s.clone(), e.clone()])
            .collect::<Vec<usize>>();

        boundaries.sort();
        boundaries.dedup();

        // \n in Spans are ignored. Therefore, we need to construct them ourselves.
        let mut lines: Vec<Line> = vec![];
        let mut current_line = Line::from("");

        // Generate styled spans as necessary.
        // TODO: Do this in a more efficient way. You can flatten the matches using
        // a stack and last known position instead of a nested lookup here.
        for window in boundaries.windows(2) {
            if let [s, e] = window
                && let Some((_, _, style)) =
                    highlights.iter().find(|(c_s, c_e, _)| c_s <= s && c_e >= e)
            {
                let fragment = &sample_text[s.clone()..e.clone()];
                for (no, fragment) in fragment.split('\n').enumerate() {
                    // This works because usually, lines are terminated with a newline, therefore,
                    // we need to prefer updating the existing line for the first split item. But,
                    // starting with the second item, we know that there was an explicit newline in
                    // between.
                    if no > 0 {
                        lines.push(current_line);
                        current_line = Line::from("");
                    }

                    if !fragment.is_empty() {
                        current_line.push_span(Span::styled(fragment.to_string(), style.clone()));
                    }
                }
            }
        }

        lines.push(current_line);
        Text::from(lines)
    }
}

fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Handle Ctrl+Q to quit
                if key.code == KeyCode::Char('q')
                    && key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    return Ok(());
                }

                // Handle Tab to switch focus.
                if matches!(key.code, KeyCode::Tab | KeyCode::BackTab) {
                    app.input_focus = match app.input_focus {
                        InputFocus::Regex => InputFocus::Sample,
                        InputFocus::Sample => InputFocus::Regex,
                    };
                    continue;
                }

                // Escape will focus the Regex field back again.
                if matches!(key.code, KeyCode::Esc) {
                    app.input_focus = InputFocus::Regex;
                    continue;
                }

                // Convert crossterm event to tui-textarea input
                let input = Input::from(Event::Key(key));

                // Handle input based on current mode
                match app.input_focus {
                    InputFocus::Regex => {
                        app.regex_textarea.input(input);
                        app.compile_regex(); // TODO: Do this in a worker thread.
                    }
                    InputFocus::Sample => {
                        app.sample_textarea.input(input);
                    }
                }
            }
        }
    }
}

// Draw the UI.
fn draw_ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Label
            Constraint::Length(1), // Regex
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Label
            Constraint::Min(8),    // Sample
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Help
        ])
        .horizontal_margin(2)
        .vertical_margin(1)
        .split(f.area());

    draw_body(f, app, (chunks[1], chunks[2], chunks[4], chunks[5]));
    draw_help(f, chunks[7]);
}

// Add a line for help text below.
fn draw_help(f: &mut ratatui::Frame, area: Rect) {
    let muted = Style::new().fg(Color::DarkGray);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            "Cycle Focus ".into(),
            Span::styled("Tab", muted),
            " ".repeat(3).into(),
            "Exit ".into(),
            Span::styled("Ctrl + q", muted),
        ])),
        area,
    );
}

// Draw the application contents.
fn draw_body(f: &mut ratatui::Frame, app: &mut App, areas: (Rect, Rect, Rect, Rect)) {
    let textarea_base = Block::default()
        .borders(Borders::LEFT)
        .border_type(BorderType::Thick)
        .border_style(Style::new().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));

    let textarea_active = textarea_base
        .clone()
        .border_style(Style::new().fg(Color::Blue));

    let textarea_error = textarea_active
        .clone()
        .fg(Color::Red);

    let cursor_active = Style::new()
        .bg(Color::White)
        .fg(Color::Black);

    let mut regex_label = Paragraph::new("Regex");
    if matches!(app.input_focus, InputFocus::Regex) {
        app.regex_textarea.set_block(match app.regex_error {
            Some(_) => textarea_error,
            None => textarea_active.clone(),
        });
        app.regex_textarea.set_cursor_style(cursor_active);
    } else {
        regex_label = regex_label.fg(Color::DarkGray);
        app.regex_textarea.set_block(match app.regex_error {
            Some(_) => textarea_error,
            None => textarea_base.clone(),
        });
        app.regex_textarea.set_cursor_style(Style::new().hidden());
    }

    let mut sample_label = Paragraph::new("Test String");
    if matches!(app.input_focus, InputFocus::Sample) {
        app.sample_textarea.set_block(textarea_active);
        app.sample_textarea.set_cursor_style(cursor_active);
    } else {
        sample_label = sample_label.fg(Color::DarkGray);
        app.sample_textarea.set_block(textarea_base.clone());
        app.sample_textarea.set_cursor_style(Style::new().hidden());
    }

    // Render the regex.
    f.render_widget(regex_label, areas.0);
    f.render_widget(&app.regex_textarea, areas.1);

    // Render the test string.
    f.render_widget(sample_label, areas.2);
    if matches!(app.input_focus, InputFocus::Sample) {
        // When focused, render the textarea for proper cursor handling.
        f.render_widget(&app.sample_textarea, areas.3);
    } else {
        // When not focused, render highlighted text.
        let highlighted_text = app.get_highlighted_text();
        let text_paragraph = Paragraph::new(highlighted_text).block(textarea_base);
        f.render_widget(text_paragraph, areas.3);
    }
}
