//! Config editor window.

use std::collections::BTreeMap;

use windows_reactor::*;

use rcm_core::files::DEFAULT_FILE;
use rcm_core::log;

use crate::visuals;

const EDITABLE_FILES: &[&str] = &["rcm.js", "rcm.config.json"];

fn file_text(text: &str) -> String {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    text.trim_end_matches('\n').to_string()
}

#[derive(Default)]
struct Buffer {
    content: String,
    baseline: String,
}

pub struct ConfigEditor {
    file: String,
    buffers: BTreeMap<String, Buffer>,
    status: String,
}

#[derive(Clone)]
pub enum Message {
    Select(String),
    Changed(String),
    Save,
    Copy,
    OpenExternal,
    Close,
}

impl ConfigEditor {
    fn content(&self) -> &str {
        self.buffers
            .get(&self.file)
            .map_or("", |buffer| buffer.content.as_str())
    }

    fn is_modified(&self, name: &str) -> bool {
        self.buffers
            .get(name)
            .is_some_and(|buffer| file_text(&buffer.content) != file_text(&buffer.baseline))
    }

    fn ensure_loaded(&mut self, name: &str) {
        if self.buffers.contains_key(name) {
            return;
        }
        if name == "rcm.js" {
            let text = rcm_core::menu::load_menu_module();
            self.buffers.insert(
                name.to_string(),
                Buffer {
                    baseline: file_text(&text),
                    content: file_text(&text),
                },
            );
            self.status = format!("Loaded {name}");
            return;
        }
        if name == "rcm.config.json" {
            rcm_core::config::init();
        }
        match rcm_core::files::read_config_file(name) {
            Ok(text) => {
                self.buffers.insert(
                    name.to_string(),
                    Buffer {
                        baseline: file_text(&text),
                        content: file_text(&text),
                    },
                );
                self.status = format!("Loaded {name}");
            }
            Err(e) => self.status = e,
        }
    }

    fn save(&mut self) {
        let name = self.file.clone();
        let Some(buffer) = self.buffers.get_mut(&name) else {
            return;
        };
        let content = file_text(&buffer.content);
        match rcm_core::files::save_config_file(&name, &content) {
            Ok(()) => {
                buffer.baseline = content;
                log::info("ConfigEditor", &format!("saved {name}"));
                self.status = format!("Saved {name}");
            }
            Err(e) => self.status = e,
        }
    }

    fn copy(&mut self) {
        let content = file_text(self.content());
        let result = rcm_core::clipboard::write_text(&content);
        self.status = match result {
            Ok(()) => format!("Copied {} to clipboard", self.file),
            Err(e) => e,
        };
    }
}

impl Component for ConfigEditor {
    type Message = Message;
    type Input = ();

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        let mut editor = Self {
            file: DEFAULT_FILE.to_string(),
            buffers: BTreeMap::new(),
            status: String::new(),
        };
        editor.ensure_loaded(DEFAULT_FILE);
        editor
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Select(name) => {
                self.file = name.clone();
                self.ensure_loaded(&name);
            }
            Message::Changed(text) => {
                let file = self.file.clone();
                if let Some(buffer) = self.buffers.get_mut(&file) {
                    buffer.content = text;
                }
            }
            Message::Save => self.save(),
            Message::Copy => self.copy(),
            Message::OpenExternal => {
                self.status = match rcm_core::files::open_config_file(&self.file) {
                    Ok(()) => format!("Opened {} in default editor", self.file),
                    Err(e) => e,
                };
            }
            Message::Close => {
                let _ = context.window().request_close();
            }
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("RCM Config Editor");
        context.window_visuals(
            WindowVisuals::new()
                .theme(visuals::window_theme())
                .backdrop(visuals::WINDOW_BACKDROP)
                .client_size(860.0, 620.0),
        );

        let file_buttons: Vec<KeyedView> = EDITABLE_FILES
            .iter()
            .map(|name| {
                let label = if self.is_modified(name) {
                    format!("● {name}")
                } else {
                    (*name).to_string()
                };
                let style = if *name == self.file {
                    ButtonStyle::Accent
                } else {
                    ButtonStyle::Default
                };
                KeyedView::new(
                    (*name).to_string(),
                    Button::new()
                        .style(style)
                        .on_click(context.message(Message::Select((*name).to_string())))
                        .content(label),
                )
            })
            .collect();

        let editor = RichEditBox::new()
            .text(self.content().to_string())
            .header(format!("Editing: {}", self.file))
            .on_text_changed(context.callback(Message::Changed));

        Border::new().padding(16.0).content(
            Grid::new()
                .rows([
                    GridLength::Auto,
                    GridLength::Star(1.0),
                    GridLength::Auto,
                    GridLength::Auto,
                ])
                .keyed_children([
                    KeyedView::new(
                        "files",
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .keyed_children(file_buttons),
                    ),
                    KeyedView::new(format!("editor-{}", self.file), editor.grid_row(1)),
                    KeyedView::new(
                        "actions",
                        StackPanel::new()
                            .orientation(Orientation::Horizontal)
                            .spacing(8.0)
                            .grid_row(2)
                            .children((
                                Button::new()
                                    .style(ButtonStyle::Accent)
                                    .on_click(context.message(Message::Save))
                                    .content("Save"),
                                Button::new()
                                    .on_click(context.message(Message::Copy))
                                    .content("Copy"),
                                Button::new()
                                    .on_click(context.message(Message::OpenExternal))
                                    .content("Open in default editor"),
                                Button::new()
                                    .on_click(context.message(Message::Close))
                                    .content("Close"),
                            )),
                    ),
                    KeyedView::new(
                        "status",
                        TextBlock::new()
                            .text(self.status.clone())
                            .opacity(0.7)
                            .grid_row(3),
                    ),
                ]),
        )
    }
}
