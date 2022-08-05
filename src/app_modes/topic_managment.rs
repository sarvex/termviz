use crate::config::{ImageListenerConfig, ListenerConfigColor, PoseListenerConfig, ListenerConfig, MapListenerConfig};
use crate::app_modes::{input, AppMode, BaseMode, Drawable};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use crate::config::TermvizConfig;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::Frame;
use tui::widgets::{Block, Borders, Paragraph, Row, ListState, Wrap, ListItem, List};


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
}

pub struct TopicManager {
    availible_topics: SelectableTopics
}

/// Represents the image view mode.
impl TopicManager {
    pub fn new() ->  TopicManager
    {
        let availible_topics = rosrust::topics()
            .unwrap()
            .iter()
            .map(|topic|{
                [topic.name.to_string(),
                 topic.datatype.to_string()
                ]
            }).collect();
        TopicManager {
            availible_topics: SelectableTopics::new(availible_topics),
        }
    }
}
impl<B: Backend> BaseMode<B> for TopicManager {}
impl AppMode for TopicManager {
    fn run(&mut self) {
    //     self.availible_topics = rosrust::topics()
    //         .unwrap()
    //         .iter()
    //         .map(|topic|{
    //             [topic.name.to_string(),
    //              topic.datatype.to_string()
    //             ]
    //         }).collect();
    }

    fn reset(&mut self, new_config: TermvizConfig) {

    }

    fn get_description(&self) -> Vec<String> {
        vec!["This mode allows to visualized images received on the given topics.".to_string()]
    }

    fn handle_input(&mut self, input: &String) {
        match input.as_str() {
            input::UP => self.availible_topics.next(),
            input::DOWN => self.availible_topics.previous(),
            _ => (),
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

        // Widget creation
        let title = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        // Conversion into tui stuff

        let items: Vec<ListItem>= self.availible_topics.items.iter().map(|i| ListItem::new(i[0].as_ref())).collect();
        // The `List` widget is then built with those items.
        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">>");;
        // Finally the widget is rendered using the associated state. `events.state` is
        // effectively the only thing that we will "remember" from this draw call.
        f.render_stateful_widget(list, f.size(), &mut self.availible_topics.state.clone());
    }
}