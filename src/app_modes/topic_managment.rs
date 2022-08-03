use crate::config::{ImageListenerConfig, ListenerConfigColor, PoseListenerConfig, ListenerConfig, MapListenerConfig};
use crate::app_modes::{input, AppMode, BaseMode, Drawable};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use crate::config::TermvizConfig;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::Frame;
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Wrap};

pub struct TopicManager {
    map_config: Vec<MapListenerConfig>,
    laser_topics: Vec<ListenerConfigColor>, 
    marker_array_topics: Vec<ListenerConfig>,
    marker_topics: Vec<ListenerConfig>,
    image_topics: Vec<ImageListenerConfig>, 
    pose_stamped_topics: Vec<PoseListenerConfig>,
    pose_array_topics: Vec<PoseListenerConfig>,
    path_topics: Vec<PoseListenerConfig>,
    
    availible_topics: Vec<[String; 2]>,
}

/// Represents the image view mode.
impl TopicManager {
    pub fn new(config: TermvizConfig) ->  TopicManager
    {
        TopicManager {
            map_config: config.map_topics,
            laser_topics: config.laser_topics,
            marker_array_topics: config.marker_array_topics,
            marker_topics: config.marker_topics,
            image_topics: config.image_topics,
            pose_stamped_topics: config.pose_stamped_topics,
            pose_array_topics: config.pose_array_topics,
            path_topics: config.path_topics,

            availible_topics: Vec::new(),
        }
    }
}
impl<B: Backend> BaseMode<B> for TopicManager {}
impl AppMode for TopicManager {
    fn run(&mut self) {
        self.availible_topics = rosrust::topics()
            .unwrap()
            .iter()
            .map(|topic|{
                [topic.name.to_string(),
                 topic.datatype.to_string()
                ]
            }).collect();
    }

    fn reset(&mut self, new_config: TermvizConfig) {

    }

    fn get_description(&self) -> Vec<String> {
        vec!["This mode allows to visualized images received on the given topics.".to_string()]
    }

    fn handle_input(&mut self, input: &String) {
    }

    fn get_keymap(&self) -> Vec<[String; 2]> {
        vec![
            [
                input::LEFT.to_string(),
                "Switches to the previous image.".to_string(),
            ],
        ]
    }

    fn get_name(&self) -> String {
        "Topic Manager".to_string()
    }
}

impl<B: Backend> Drawable<B> for TopicManager {
    fn draw(&self, f: &mut Frame<B>) {
        // Text
        let title_text = vec![Spans::from(Span::styled(
            "Topic Manager",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))];
        // Define areas from text
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(20)
            .constraints(
                [
                    Constraint::Length(3), // Title + 2 borders
                    Constraint::Length(2),
                    Constraint::Min(1), // Table + header + space
                ]
                .as_ref(),
            )
            .split(f.size());

        // Conversion into tui stuff
        let key_bindings_rows = self.availible_topics.clone().into_iter().map(|x| Row::new(x));

        // Widget creation
        let title = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        let key_bindings = Table::new(IntoIterator::into_iter(key_bindings_rows))
            .block(
                Block::default()
                    .title(" Key binding ")
                    .borders(Borders::ALL),
            )
            .header(Row::new(vec!["Key", "Function"]).style(Style::default().fg(Color::Yellow)))
            .widths(&[Constraint::Min(9), Constraint::Percentage(100)])
            .style(Style::default().fg(Color::White))
            .column_spacing(10);
        f.render_widget(title, areas[0]);
        f.render_widget(key_bindings, areas[2]);
    }
}