use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListDirection;
use ratatui::widgets::Paragraph;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

pub struct Ui<Context> {
    input: Input,

    pub max_height_percent: u16,
    _pd: std::marker::PhantomData<fn(Context)>,
}

impl<Context> Default for Ui<Context> {
    fn default() -> Self {
        Self {
            input: Input::default(),
            max_height_percent: 50,
            _pd: std::marker::PhantomData,
        }
    }
}

impl<Context> Ui<Context> {
    pub fn handle_key_press(&mut self, event: KeyEvent) {
        self.input.handle_event(&Event::Key(event));
    }

    pub fn value(&self) -> &str {
        self.input.value()
    }

    #[inline]
    pub fn reset_value(&mut self) {
        self.input.reset()
    }
}

impl<Context> StatefulWidget for &mut Ui<Context>
where
    Context: crate::Context + Sized,
{
    type State = crate::Commander<Context>;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        if state.is_active() {
            let msg = vec![
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit prompt"),
            ];
            let style = Style::default().add_modifier(Modifier::RAPID_BLINK);

            let suggestions = state.suggestions();

            let suggestions_height = self.max_height_percent.min({
                suggestions.len() as u16 + 2 // borders
            });

            let [_rest, commander_area] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(suggestions_height + 3),
            ])
            .flex(ratatui::layout::Flex::Start)
            .areas(area);

            let [suggestions_area, command_area] = Layout::vertical([
                Constraint::Length(suggestions_height),
                Constraint::Length(3),
            ])
            .areas(commander_area);

            if !suggestions.is_empty() {
                ratatui::widgets::Clear.render(suggestions_area, buf);
                let list = List::new(suggestions.clone())
                    .block(Block::bordered().title("Suggestions"))
                    .style(Style::new().white())
                    .highlight_style(Style::new().italic())
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true)
                    .direction(ListDirection::BottomToTop);
                Widget::render(list, suggestions_area, buf)
            }

            let [commander_logo_area, inserting_area, desc_area] = Layout::horizontal([
                Constraint::Min(3),
                Constraint::Percentage(100),
                Constraint::Min(20),
            ])
            .areas(command_area);

            let logo = Paragraph::new(
                Text::from(Line::from(Span::styled(
                    ":",
                    Style::default().add_modifier(Modifier::BOLD),
                )))
                .style(style),
            )
            .block(Block::default().borders(Borders::ALL));

            let desc_text = Paragraph::new(Text::from(Line::from(msg)).style(style))
                .block(Block::default().borders(Borders::ALL));

            let input = Paragraph::new(self.input.value())
                .style(
                    if state.is_unknown_command() || !state.current_args_are_valid().unwrap_or(true)
                    {
                        Style::default().on_red()
                    } else {
                        Style::default()
                    },
                )
                .block(Block::default().borders(Borders::ALL).title("Input"));

            ratatui::widgets::Clear.render(command_area, buf);
            logo.render(commander_logo_area, buf);
            input.render(inserting_area, buf);
            desc_text.render(desc_area, buf);
        }
    }
}
