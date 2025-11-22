use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::text::Line;
use ratatui::widgets::Widget;

pub struct CommanderView {
    input_top: bool,
    list_widget_builder: Box<dyn Fn(Vec<String>) -> ratatui::widgets::List<'static>>,
    input_line_processing: Option<Box<dyn Fn(Line<'_>) -> Line<'_>>>,
}

impl CommanderView {
    pub fn new(
        list_widget_builder: Box<dyn Fn(Vec<String>) -> ratatui::widgets::List<'static>>,
    ) -> Self {
        Self {
            input_top: false,
            list_widget_builder,
            input_line_processing: None,
        }
    }

    pub fn with_input_line_processing<P>(mut self, proc: P) -> Self
    where
        P: Fn(Line<'_>) -> Line<'_>,
        P: 'static,
    {
        self.input_line_processing = Some(Box::new(proc));
        self
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
        let line = ratatui::text::Line::from(state.input());

        let line = if let Some(proc) = self.input_line_processing.as_ref() {
            (proc)(line)
        } else {
            line
        };

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
