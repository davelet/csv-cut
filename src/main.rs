use humansize::{format_size, DECIMAL};
use iced::font::Family;
use iced::widget::{button, checkbox, column, container, text, Container, Scrollable};
use iced::{Application, Background, Color, Command, Element, Font, Length, Settings, Theme};
use rfd::FileDialog;
use std::path::PathBuf;

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
    SelectAll,
    InvertSelection,
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
                if let Some(path) = FileDialog::new().add_filter("CSV", &["csv"]).pick_file() {
                    return Command::perform(async move { path }, Message::FileSelected);
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
                if let Some(input_path) = &self.csv_path {
                    if let Some(output_path) =
                        FileDialog::new().add_filter("CSV", &["csv"]).save_file()
                    {
                        if let Ok(input_file) = std::fs::File::open(input_path) {
                            let mut reader = csv::Reader::from_reader(input_file);
                            if let Ok(output_file) = std::fs::File::create(output_path) {
                                let mut writer = csv::Writer::from_writer(output_file);

                                // Write selected headers
                                let selected_headers: Vec<String> = self
                                    .columns
                                    .iter()
                                    .filter(|(_, selected)| *selected)
                                    .map(|(name, _)| name.clone())
                                    .collect();

                                if let Err(err) = writer.write_record(&selected_headers) {
                                    self.error_message =
                                        Some(format!("Failed to write headers: {}", err));
                                    return Command::none();
                                }

                                // Write selected columns for each record
                                for result in reader.records() {
                                    match result {
                                        Ok(record) => {
                                            let selected_fields: Vec<String> = self
                                                .columns
                                                .iter()
                                                .enumerate()
                                                .filter(|(_, (_, selected))| *selected)
                                                .map(|(i, _)| {
                                                    record.get(i).unwrap_or("").to_string()
                                                })
                                                .collect();

                                            if let Err(err) = writer.write_record(&selected_fields)
                                            {
                                                self.error_message = Some(format!(
                                                    "Failed to write record: {}",
                                                    err
                                                ));
                                                return Command::none();
                                            }
                                        }
                                        Err(err) => {
                                            self.error_message =
                                                Some(format!("Failed to read record: {}", err));
                                            return Command::none();
                                        }
                                    }
                                }

                                if let Err(err) = writer.flush() {
                                    self.error_message =
                                        Some(format!("Failed to flush writer: {}", err));
                                    return Command::none();
                                }
                            } else {
                                self.error_message =
                                    Some("Failed to create output file".to_string());
                            }
                        } else {
                            self.error_message = Some("Failed to open input file".to_string());
                        }
                    }
                }
                Command::none()
            }
            Message::SelectAll => {
                for (_, selected) in self.columns.iter_mut() {
                    *selected = true;
                }
                Command::none()
            }
            Message::InvertSelection => {
                for (_, selected) in self.columns.iter_mut() {
                    *selected = !*selected;
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let mut content = column![]
            .spacing(20)
            .padding(20)
            .push(button("选择CSV文件").on_press(Message::OpenFile));

        if let Some(path) = &self.csv_path {
            content = content.push(text(format!("文件: {}", path.display())));

            if let Some(size) = self.file_size {
                content = content.push(text(format!("大小: {}", format_size(size, DECIMAL))));
            }

            if let Some(rows) = self.total_rows {
                content = content.push(text(format!("行数: {}", rows)));
            }

            let columns = column(
                self.columns
                    .iter()
                    .enumerate()
                    .filter(|(_, (name, _))| !name.is_empty())
                    .map(|(i, (name, selected))| {
                        checkbox(name.clone(), *selected)
                            .on_toggle(move |_| Message::ToggleColumn(i))
                            .into()
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(10);

            let scrollable_columns = Scrollable::new(
                Container::new(columns)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .style(|_theme: &iced::Theme| container::Appearance {
                        background: Some(Background::Color(Color::from_rgb8(0xFF, 0xE4, 0xC4))), // Example color
                        ..Default::default()
                    }),
            )
            .height(Length::Fixed(300.0));

            content = content
                .push(text("选择要保留的列:"))
                .push(scrollable_columns)
                .push(
                    iced::widget::row![]
                        .spacing(10)
                        .push(button("全选").on_press(Message::SelectAll))
                        .push(button("反选").on_press(Message::InvertSelection))
                        .push(button("保存").on_press(Message::SaveFile)),
                );
        }

        if let Some(error) = &self.error_message {
            content = content.push(text(error).style(iced::theme::Text::Color(
                iced::Color::from_rgb(1.0, 0.0, 0.0),
            )));
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
