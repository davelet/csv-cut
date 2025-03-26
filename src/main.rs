use iced::font::Family;
use iced::{Application, Command, Element, Font, Length, Settings, Theme};
use iced::widget::{button, checkbox, column, container, text};
use rfd::FileDialog;
use std::path::PathBuf;
use humansize::{format_size, DECIMAL};

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    let font = Font {
        family: Family::Name("PingFang SC"),
        ..Font::default()
    };
    settings.default_font = font;
    CsvCutter::run(settings)
}

#[derive(Debug, Default)]
struct CsvCutter {
    csv_path: Option<PathBuf>,
    file_size: Option<u64>,
    total_rows: Option<usize>,
    columns: Vec<(String, bool)>,
    error_message: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFile,
    FileSelected(PathBuf),
    ToggleColumn(usize),
    SaveFile,
}

impl Application for CsvCutter {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("CSV Cutter")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::OpenFile => {
                if let Some(path) = FileDialog::new()
                    .add_filter("CSV", &["csv"])
                    .pick_file() {
                    return Command::perform(
                        async move { path },
                        Message::FileSelected,
                    );
                }
                Command::none()
            }
            Message::FileSelected(path) => {
                self.csv_path = Some(path.clone());
                if let Ok(file) = std::fs::File::open(&path) {
                    if let Ok(metadata) = file.metadata() {
                        self.file_size = Some(metadata.len());
                    }
                    
                    let mut reader = csv::Reader::from_reader(file);
                    if let Ok(headers) = reader.headers() {
                        self.columns = headers
                            .iter()
                            .map(|h| (h.to_string(), true))
                            .collect::<Vec<_>>();
                        
                        self.total_rows = reader.records().count().checked_sub(1);
                    } else {
                        self.error_message = Some("Failed to read CSV headers".to_string());
                    }
                } else {
                    self.error_message = Some("Failed to open file".to_string());
                }
                Command::none()
            }
            Message::ToggleColumn(index) => {
                if let Some((_, selected)) = self.columns.get_mut(index) {
                    *selected = !*selected;
                }
                Command::none()
            }
            Message::SaveFile => {
                // TODO: Implement save functionality
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let mut content = column![]
            .spacing(20)
            .padding(20)
            .push(
                button("选择CSV文件")
                    .on_press(Message::OpenFile)
            );

        if let Some(path) = &self.csv_path {
            content = content.push(
                text(format!("文件: {}", path.display()))
            );

            if let Some(size) = self.file_size {
                content = content.push(
                    text(format!("大小: {}", format_size(size, DECIMAL)))
                );
            }

            if let Some(rows) = self.total_rows {
                content = content.push(
                    text(format!("行数: {}", rows))
                );
            }

            let columns = column(self.columns.iter().enumerate().map(|(i, (name, selected))| {
                checkbox(
                    name.clone(),
                    *selected
                ).on_toggle(move |_| Message::ToggleColumn(i)).into()
            }).collect::<Vec<_>>())
            .spacing(10);

            content = content
                .push(text("选择要保留的列:"))
                .push(columns)
                .push(
                    button("保存")
                        .on_press(Message::SaveFile)
                );
        }

        if let Some(error) = &self.error_message {
            content = content.push(
                text(error).style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.0, 0.0)))
            );
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}