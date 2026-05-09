use crate::{app, config, doctor, lang::Language};
use anyhow::{anyhow, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    DefaultTerminal,
};
use std::io::{self, IsTerminal};
use std::time::Duration;

const MENU_LEN: usize = 8;
const CREATE_SLOT_COUNT: usize = 8;

#[derive(Clone, Copy)]
enum PendingAction {
    CreateSelect,
    OpenSelect,
    RemoveSelect,
    LanguageSelect,
    CreateForceConfirm(u16),
    RemoveConfirm(u16),
}

#[derive(Clone, Copy)]
enum ConfirmAction {
    CreateForce(u16),
    Remove(u16),
}

#[derive(Clone, Copy)]
enum OutputTone {
    Normal,
    Success,
    Error,
}

pub fn run(language: Language, should_prompt_for_language: bool) -> Result<()> {
    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
        return Err(anyhow!(language.tui_requires_terminal()));
    }

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = ratatui::init();
    let result = run_loop(
        &mut terminal,
        AppState::new(language, should_prompt_for_language),
    );
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    ratatui::restore();
    result
}

fn run_loop(terminal: &mut DefaultTerminal, mut app_state: AppState) -> Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, &app_state))?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        if app_state.handle_key(key.code)? {
            return Ok(());
        }
    }
}

struct AppState {
    language: Language,
    selected: usize,
    output_title: String,
    output: String,
    output_tone: OutputTone,
    pending: Option<PendingAction>,
    create_candidates: Vec<u16>,
    create_selected: usize,
    open_candidates: Vec<app::AppInstance>,
    open_selected: usize,
    remove_candidates: Vec<app::AppInstance>,
    remove_selected: usize,
    confirm_action: Option<ConfirmAction>,
    confirm_selected: usize,
    language_selected: usize,
}

impl AppState {
    fn new(language: Language, should_prompt_for_language: bool) -> Self {
        Self {
            language,
            selected: 0,
            output_title: language.tui_output_welcome_title().to_string(),
            output: language.tui_welcome().to_string(),
            output_tone: OutputTone::Normal,
            pending: if should_prompt_for_language {
                Some(PendingAction::LanguageSelect)
            } else {
                None
            },
            create_candidates: Vec::new(),
            create_selected: 0,
            open_candidates: Vec::new(),
            open_selected: 0,
            remove_candidates: Vec::new(),
            remove_selected: 0,
            confirm_action: None,
            confirm_selected: 1,
            language_selected: if matches!(language, Language::Zh) {
                0
            } else {
                1
            },
        }
    }

    fn handle_key(&mut self, code: KeyCode) -> Result<bool> {
        if let Some(pending) = self.pending {
            return self.handle_pending_key(code, pending);
        }

        match code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1).min(MENU_LEN - 1)
            }
            KeyCode::Enter => return self.activate_selected(),
            KeyCode::Char('1') => self.run_list(),
            KeyCode::Char('2') => self.start_create(),
            KeyCode::Char('3') => self.start_open_specific(),
            KeyCode::Char('4') => self.run_open_all(),
            KeyCode::Char('5') => self.start_remove(),
            KeyCode::Char('6') => self.run_doctor(),
            KeyCode::Char('0') => return Ok(true),
            _ => {}
        }

        Ok(false)
    }

    fn handle_pending_key(&mut self, code: KeyCode, pending: PendingAction) -> Result<bool> {
        match pending {
            PendingAction::CreateSelect => self.handle_create_select_key(code),
            PendingAction::OpenSelect => self.handle_open_select_key(code),
            PendingAction::RemoveSelect => self.handle_remove_select_key(code),
            PendingAction::LanguageSelect => self.handle_language_select_key(code),
            PendingAction::CreateForceConfirm(index) => {
                self.handle_confirm_key(code, ConfirmAction::CreateForce(index))
            }
            PendingAction::RemoveConfirm(index) => {
                self.handle_confirm_key(code, ConfirmAction::Remove(index))
            }
        }
    }

    fn handle_create_select_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc => {
                self.pending = None;
                self.create_candidates.clear();
                self.create_selected = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.create_selected = self.create_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.create_candidates.is_empty() {
                    self.create_selected =
                        (self.create_selected + 1).min(self.create_candidates.len() - 1);
                }
            }
            KeyCode::Enter => {
                if let Some(index) = self.create_candidates.get(self.create_selected).copied() {
                    self.pending = Some(PendingAction::CreateForceConfirm(index));
                    self.confirm_action = Some(ConfirmAction::CreateForce(index));
                    self.confirm_selected = 1;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_open_select_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc => {
                self.pending = None;
                self.open_candidates.clear();
                self.open_selected = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.open_selected = self.open_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.open_candidates.is_empty() {
                    self.open_selected =
                        (self.open_selected + 1).min(self.open_candidates.len() - 1);
                }
            }
            KeyCode::Enter => {
                if let Some(instance) = self.open_candidates.get(self.open_selected) {
                    self.pending = None;
                    let index = instance.index;
                    self.open_candidates.clear();
                    self.open_selected = 0;
                    self.push_result(
                        self.language.tui_output_open_title(),
                        app::open_instance(self.language, index),
                    );
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_remove_select_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc => {
                self.pending = None;
                self.remove_candidates.clear();
                self.remove_selected = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.remove_selected = self.remove_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.remove_candidates.is_empty() {
                    self.remove_selected =
                        (self.remove_selected + 1).min(self.remove_candidates.len() - 1);
                }
            }
            KeyCode::Enter => {
                if let Some(instance) = self.remove_candidates.get(self.remove_selected) {
                    let index = instance.index;
                    self.pending = Some(PendingAction::RemoveConfirm(index));
                    self.confirm_action = Some(ConfirmAction::Remove(index));
                    self.confirm_selected = 1;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_language_select_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc => {
                self.pending = None;
            }
            KeyCode::Up | KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('h') => {
                self.language_selected = self.language_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('l') => {
                self.language_selected = (self.language_selected + 1).min(1);
            }
            KeyCode::Enter => {
                let language = if self.language_selected == 0 {
                    Language::Zh
                } else {
                    Language::En
                };
                self.apply_language(language);
                match config::save_language(language) {
                    Ok(()) => self.set_notice(self.language.tui_language_saved().to_string()),
                    Err(err) => {
                        self.set_notice(self.language.tui_language_save_failed(&err.to_string()))
                    }
                }
                self.pending = None;
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_confirm_key(&mut self, code: KeyCode, action: ConfirmAction) -> Result<bool> {
        match code {
            KeyCode::Esc => {
                self.confirm_action = None;
                self.pending = None;
                self.confirm_selected = 1;
                self.create_candidates.clear();
                self.open_candidates.clear();
                self.remove_candidates.clear();
            }
            KeyCode::Left | KeyCode::Up | KeyCode::Char('h') | KeyCode::Char('k') => {
                self.confirm_selected = self.confirm_selected.saturating_sub(1);
            }
            KeyCode::Right | KeyCode::Down | KeyCode::Char('l') | KeyCode::Char('j') => {
                self.confirm_selected = (self.confirm_selected + 1).min(1);
            }
            KeyCode::Enter => {
                let confirmed = self.confirm_selected == 0;
                self.confirm_action = None;
                self.pending = None;
                self.confirm_selected = 1;

                match action {
                    ConfirmAction::CreateForce(index) => {
                        self.create_candidates.clear();
                        self.create_selected = 0;
                        self.push_result(
                            self.language.tui_output_create_title(),
                            app::create_instance(self.language, index, confirmed),
                        );
                    }
                    ConfirmAction::Remove(index) => {
                        self.remove_candidates.clear();
                        self.remove_selected = 0;
                        if confirmed {
                            self.push_result(
                                self.language.tui_output_remove_title(),
                                app::remove_instance(self.language, index, true),
                            );
                        } else {
                            self.set_notice(self.language.removal_cancelled().to_string());
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn activate_selected(&mut self) -> Result<bool> {
        match self.selected {
            0 => self.run_list(),
            1 => self.start_create(),
            2 => self.start_open_specific(),
            3 => self.run_open_all(),
            4 => self.start_remove(),
            5 => self.run_doctor(),
            6 => self.start_language_switch(),
            7 => return Ok(true),
            _ => {}
        }

        Ok(false)
    }

    fn start_create(&mut self) {
        match app::suggested_available_indices(CREATE_SLOT_COUNT) {
            Ok(indices) if indices.is_empty() => {
                self.set_notice(self.language.no_local_copies().to_string());
            }
            Ok(indices) => {
                self.create_candidates = indices;
                self.create_selected = 0;
                self.pending = Some(PendingAction::CreateSelect);
            }
            Err(err) => self.set_error(err.to_string()),
        }
    }

    fn start_open_specific(&mut self) {
        match app::scan_copies() {
            Ok(copies) if copies.is_empty() => {
                self.set_notice(self.language.tui_no_copies_to_open().to_string());
            }
            Ok(copies) => {
                self.open_candidates = copies;
                self.open_selected = 0;
                self.pending = Some(PendingAction::OpenSelect);
            }
            Err(err) => self.set_error(err.to_string()),
        }
    }

    fn start_remove(&mut self) {
        match app::scan_copies() {
            Ok(copies) if copies.is_empty() => {
                self.set_notice(self.language.tui_no_copies_to_remove().to_string());
            }
            Ok(copies) => {
                self.remove_candidates = copies;
                self.remove_selected = 0;
                self.pending = Some(PendingAction::RemoveSelect);
            }
            Err(err) => self.set_error(err.to_string()),
        }
    }

    fn start_language_switch(&mut self) {
        self.language_selected = if matches!(self.language, Language::Zh) {
            0
        } else {
            1
        };
        self.pending = Some(PendingAction::LanguageSelect);
    }

    fn run_list(&mut self) {
        self.push_result(
            self.language.tui_output_list_title(),
            app::list_instances(self.language),
        );
    }

    fn run_open_all(&mut self) {
        self.push_result(
            self.language.tui_output_open_title(),
            app::open_all(self.language),
        );
    }

    fn run_doctor(&mut self) {
        self.push_result(
            self.language.tui_output_doctor_title(),
            doctor::run(self.language),
        );
    }

    fn push_result(&mut self, title: &str, result: Result<String>) {
        match result {
            Ok(text) => self.set_success(title.to_string(), text),
            Err(err) => self.set_error(err.to_string()),
        }
    }

    fn set_success(&mut self, title: String, text: String) {
        self.output_title = title;
        self.output = text;
        self.output_tone = OutputTone::Success;
    }

    fn set_notice(&mut self, text: String) {
        self.output_title = self.language.tui_output_notice_title().to_string();
        self.output = text;
        self.output_tone = OutputTone::Normal;
    }

    fn set_error(&mut self, text: String) {
        self.output_title = self.language.tui_output_error_title().to_string();
        self.output = format!("{}: {}", self.language.error_label(), text);
        self.output_tone = OutputTone::Error;
    }

    fn apply_language(&mut self, language: Language) {
        self.language = language;
        if matches!(self.output_tone, OutputTone::Normal)
            && self.output_title == self.language.tui_output_welcome_title()
        {
            self.output = self.language.tui_welcome().to_string();
        }
    }
}

fn draw(frame: &mut Frame, app: &AppState) {
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(app.language.tui_title())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD)),
        vertical[0],
    );

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(vertical[1]);

    let items = app
        .language
        .tui_menu_items()
        .into_iter()
        .map(ListItem::new)
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select(Some(app.selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(
                Block::default()
                    .title(app.language.tui_actions())
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> "),
        horizontal[0],
        &mut state,
    );

    let output_title_style = match app.output_tone {
        OutputTone::Normal => Style::default().add_modifier(Modifier::BOLD),
        OutputTone::Success => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        OutputTone::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    };
    let output_block = Block::default()
        .title(Line::from(vec![
            Span::raw(format!("{} | ", app.language.tui_output())),
            Span::styled(app.output_title.as_str(), output_title_style),
        ]))
        .borders(Borders::ALL);

    frame.render_widget(
        Paragraph::new(app.output.as_str())
            .block(output_block)
            .wrap(Wrap { trim: false }),
        horizontal[1],
    );

    frame.render_widget(
        Paragraph::new(app.language.tui_help()).block(Block::default().borders(Borders::ALL)),
        vertical[2],
    );

    if let Some(pending) = app.pending {
        draw_modal(frame, area, app, pending);
    }
}

fn draw_modal(frame: &mut Frame, area: Rect, app: &AppState, pending: PendingAction) {
    match pending {
        PendingAction::CreateSelect => draw_create_select_modal(frame, area, app),
        PendingAction::OpenSelect => draw_open_select_modal(frame, area, app),
        PendingAction::RemoveSelect => draw_remove_select_modal(frame, area, app),
        PendingAction::LanguageSelect => draw_language_select_modal(frame, area, app),
        PendingAction::CreateForceConfirm(index) => {
            draw_confirm_modal(frame, area, app, app.language.create_force(), Some(index))
        }
        PendingAction::RemoveConfirm(index) => draw_confirm_modal(
            frame,
            area,
            app,
            &app.language
                .removal_prompt(&app::app_path(index).display().to_string()),
            None,
        ),
    }
}

fn draw_language_select_modal(frame: &mut Frame, area: Rect, app: &AppState) {
    let popup = centered_rect(50, 30, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .title(app.language.tui_language_title())
        .borders(Borders::ALL);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(inner);

    let items = app
        .language
        .tui_language_choices()
        .into_iter()
        .map(ListItem::new)
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select(Some(app.language_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> "),
        chunks[0],
        &mut state,
    );
    frame.render_widget(Paragraph::new(app.language.tui_language_help()), chunks[1]);
}

fn draw_create_select_modal(frame: &mut Frame, area: Rect, app: &AppState) {
    let popup = centered_rect(60, 40, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .title(app.language.tui_create_title())
        .borders(Borders::ALL);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(inner);

    let items = app
        .create_candidates
        .iter()
        .map(|index| ListItem::new(format!("WeChat{index}.app")))
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select(Some(app.create_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> "),
        chunks[0],
        &mut state,
    );
    frame.render_widget(
        Paragraph::new(app.language.tui_create_select_help()),
        chunks[1],
    );
}

fn draw_open_select_modal(frame: &mut Frame, area: Rect, app: &AppState) {
    let popup = centered_rect(60, 40, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .title(app.language.tui_open_title())
        .borders(Borders::ALL);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(inner);

    let items = app
        .open_candidates
        .iter()
        .map(|instance| {
            ListItem::new(format!(
                "WeChat{}  {}",
                instance.index,
                instance.path.display()
            ))
        })
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select(Some(app.open_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> "),
        chunks[0],
        &mut state,
    );
    frame.render_widget(
        Paragraph::new(app.language.tui_open_select_help()),
        chunks[1],
    );
}

fn draw_remove_select_modal(frame: &mut Frame, area: Rect, app: &AppState) {
    let popup = centered_rect(60, 40, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .title(app.language.tui_remove_title())
        .borders(Borders::ALL);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(inner);

    let items = app
        .remove_candidates
        .iter()
        .map(|instance| {
            ListItem::new(format!(
                "WeChat{}  {}",
                instance.index,
                instance.path.display()
            ))
        })
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select(Some(app.remove_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> "),
        chunks[0],
        &mut state,
    );
    frame.render_widget(
        Paragraph::new(app.language.tui_remove_select_help()),
        chunks[1],
    );
}

fn draw_confirm_modal(
    frame: &mut Frame,
    area: Rect,
    app: &AppState,
    prompt: &str,
    index_hint: Option<u16>,
) {
    let popup = centered_rect(60, 24, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .title(app.language.tui_confirm_title())
        .borders(Borders::ALL);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(inner);

    let text = if let Some(index) = index_hint {
        format!("{prompt} [{}]", app::app_path(index).display())
    } else {
        prompt.to_string()
    };
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), chunks[0]);

    let choice_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let yes_style = if app.confirm_selected == 0 {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
    } else {
        Style::default()
    };
    let no_style = if app.confirm_selected == 1 {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
    } else {
        Style::default()
    };

    frame.render_widget(
        Paragraph::new(app.language.tui_confirm_yes())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(yes_style),
        choice_chunks[0],
    );
    frame.render_widget(
        Paragraph::new(app.language.tui_confirm_no())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
            .style(no_style),
        choice_chunks[1],
    );
    frame.render_widget(Paragraph::new(app.language.tui_confirm_help()), chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
