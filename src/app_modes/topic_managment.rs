use std::sync::{Arc, RwLock, atomic};

use crate::config::{ImageListenerConfig, ListenerConfigColor, PoseListenerConfig, ListenerConfig, MapListenerConfig};
use crate::config::Color as ConfigColor;
use crate::app_modes::{input, AppMode, BaseMode, Drawable, ExitCode};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use crate::config::TermvizConfig;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::Frame;
use tui::widgets::{Block, Borders, Paragraph, ListState, Wrap, ListItem, List};


#[derive(Clone)]
struct SelectableTopics {
    // `items` is the state managed by your application.
    items: Vec<[String; 2]>,
    // `state` is the state that can be modified by the UI. It stores the index of the selected
    // item as well as the offset computed during the previous draw call (used to implement
    // natural scrolling).
    state: ListState,
}

impl SelectableTopics {
    fn new(items: Vec<[String; 2]>) -> SelectableTopics {
        SelectableTopics {
            items,
            state: ListState::default(),
        }
    }


    pub fn set_items(&mut self, items: Vec<[String; 2]>) {
        self.items = items;
        // We reset the state as the associated items have changed. This effectively reset
        // the selection as well as the stored offset.
        self.state = ListState::default();
    }

    // Select the next item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // Select the previous item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // Unselect the currently selected item if any. The implementation of `ListState` makes
    // sure that the stored offset is also reset.
    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn add(&mut self, element: [String; 2]) {
        self.items.push(element);
    }

    pub fn pop(&mut self) -> [String; 2]{
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.items.remove(i)
    }
}

pub struct TopicManager {
    availible_topics: SelectableTopics,
    selected_topics: SelectableTopics,
    shared_config: Arc<RwLock<TermvizConfig>>,
    selection_mode: bool,
    reset_requested: Arc<atomic::AtomicBool>,
}

/// Represents the image view mode.
impl TopicManager {
    pub fn new(shared_config: Arc<RwLock<TermvizConfig>>) ->  TopicManager
    {
        let config = shared_config.read().unwrap().clone();
        let availible_topics: Vec<[String; 2]>= rosrust::topics()
            .unwrap()
            .iter()
            .map(|topic|{
                [topic.name.to_string(),
                 topic.datatype.to_string()
                ]
            }).collect();

        let mut supported_topics: Vec<[String; 2]> = Vec::new();

        for topic in availible_topics.into_iter() {
            if topic[0].to_string() != "sensor_msgs/Laserscan"{
                supported_topics.push(topic.clone());

            }
        }

        let all_active_topics: Vec<[String; 2]>= vec![
            [config.laser_topics.iter().map(|i| i.topic.clone()).collect(), "sensor_msgs/LaserScan".to_string()],
            [config.marker_array_topics.iter().map(|i| i.topic.clone()).collect(), "visualization_msgs/MarkerArray".to_string()],
            [config.marker_topics.iter().map(|i| i.topic.clone()).collect(), "visualization_msgs/Marker".to_string()],
            [config.pose_stamped_topics.iter().map(|i| i.topic.clone()).collect(), "geometry_msgs/PoseStamped".to_string()],
            [config.pose_array_topics.iter().map(|i| i.topic.clone()).collect(), "geometry_msgs/PoseArray".to_string()],
            [config.path_topics.iter().map(|i| i.topic.clone()).collect(), "nav_msgs/Path".to_string()],
        ];

        TopicManager {
            availible_topics: SelectableTopics::new(supported_topics),
            selected_topics: SelectableTopics::new(all_active_topics),
            shared_config: shared_config,
            selection_mode: true,
            reset_requested: Arc::new(atomic::AtomicBool::new(false))
        }
    }

    pub fn shift_active_element_right(&mut self) {
        let x = self.availible_topics.pop();
        self.selected_topics.add(x);
    }
    pub fn shift_active_element_left(&mut self) {
        let x = self.selected_topics.pop();
        self.availible_topics.add(x);
    }

    pub fn save(&mut self) {
        for topic in self.selected_topics.items.iter() {
            match topic[1].clone().as_ref() {
                "sensor_msgs/LaserScan" => self.shared_config.clone().write().unwrap().laser_topics.push(
                    ListenerConfigColor{
                        topic: topic[0].clone(),
                        color: ConfigColor{r:255, g:255, b:255},
                    }),
                _ => (),
            }
        }
        self.reset_requested.store(false, atomic::Ordering::Relaxed)
    }

}

impl<B: Backend> BaseMode<B> for TopicManager {}

impl AppMode for TopicManager {
    fn run(&mut self) -> ExitCode{
        if self.reset_requested.swap(false, atomic::Ordering::Relaxed) {
            return ExitCode::Reset;
        }
        return ExitCode::Noop;
    }

    fn reset(&mut self, new_config: TermvizConfig) {

    }

    fn get_description(&self) -> Vec<String> {
        vec!["This mode allows to visualized images received on the given topics.".to_string()]
    }

    fn handle_input(&mut self, input: &String) {
        if self.selection_mode {
            match input.as_str() {
                input::UP => self.availible_topics.previous(),
                input::DOWN => self.availible_topics.next(),
                input::RIGHT => self.shift_active_element_right(),
                input::ROTATE_RIGHT => self.selection_mode = false,
                input::CONFIRM => self.save(),
                _ => (),
            }
        }
        else {
            match input.as_str() {
                input::UP => self.selected_topics.previous(),
                input::DOWN => self.selected_topics.next(),
                input::LEFT => self.shift_active_element_left(),
                input::ROTATE_LEFT => self.selection_mode = true,
                input::CONFIRM => self.save(),
                _ => (),
            }
        }
    }

    fn get_keymap(&self) -> Vec<[String; 2]> {
        vec![
            [
                input::UP.to_string(),
                "Shifts the pose estimate positively along the x axis.".to_string(),
            ],
            [
                input::DOWN.to_string(),
                "Shifts the pose estimate negatively along the x axis.".to_string(),
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

        let left_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(areas[2]);
        let right_pane =
            Block::default().title("BlockR").borders(Borders::ALL);

        // Widget creation
        let title = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        // Conversion into tui stuff

        let items: Vec<ListItem>= self.availible_topics.items.iter().map(|i| ListItem::new(i[1].as_ref())).collect();
        // The `List` widget is then built with those items.
        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .block( Block::default().title("Availible Topics").borders(Borders::ALL))
            .highlight_symbol(">>");

        let selected_items: Vec<ListItem>= self.selected_topics.items.iter().map(|i| ListItem::new(i[0].as_ref())).collect();
        // The `List` widget is then built with those items.
        let selected_list = List::new(selected_items)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .block( Block::default().title("Availible Topics").borders(Borders::ALL))
            .highlight_symbol(">>");
        // Finally the widget is rendered using the associated state. `events.state` is
        // effectively the only thing that we will "remember" from this draw call.
        f.render_widget(title, areas[0]);
        f.render_stateful_widget(list, left_chunks[0], &mut self.availible_topics.state.clone());
        f.render_stateful_widget(selected_list, left_chunks[1], &mut self.selected_topics.state.clone());
    }
}