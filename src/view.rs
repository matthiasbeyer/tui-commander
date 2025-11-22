use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::text::Line;
use ratatui::widgets::Widget;

pub struct CommanderView {
    input_top: bool,
    list_widget_builder: Box<dyn Fn(Vec<String>) -> ratatui::widgets::List<'static>>,
    input_line_widget_builder: Box<dyn Fn(String) -> Line<'static>>,
}

impl CommanderView {
    pub fn new(
        input_line_widget_builder: Box<dyn Fn(String) -> Line<'static>>,
        list_widget_builder: Box<dyn Fn(Vec<String>) -> ratatui::widgets::List<'static>>,
    ) -> Self {
        Self {
            input_top: false,
            list_widget_builder,
            input_line_widget_builder,
        }
    }
}

impl ratatui::widgets::StatefulWidget for CommanderView {
    type State = crate::Commander;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let [input_area, suggestions_area] = if self.input_top {
            let [input_area, suggestions_area] =
                Layout::vertical(vec![Constraint::Min(1), Constraint::Percentage(100)]).areas(area);
            [input_area, suggestions_area]
        } else {
            let [suggestions_area, input_area] =
                Layout::vertical(vec![Constraint::Percentage(100), Constraint::Min(1)]).areas(area);

            [input_area, suggestions_area]
        };

        self.render_input(input_area, buf, state);
        self.render_suggestions(suggestions_area, buf, state);
    }
}

impl CommanderView {
    fn render_input(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut crate::Commander,
    ) {
        let line = (self.input_line_widget_builder)(state.input().to_string());
        line.render(area, buf);
    }

    fn render_suggestions(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut crate::Commander,
    ) {
        let list = (self.list_widget_builder)(state.suggestions());
        ratatui::widgets::StatefulWidget::render(
            list,
            area,
            buf,
            state.suggestion_list_state_mut(),
        );
    }
}
