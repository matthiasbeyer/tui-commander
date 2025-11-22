use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::text::Line;
use ratatui::widgets::Widget;

pub struct CommanderView<'l> {
    pub input_top: bool,
    pub input_line_processing: &'l dyn LineProcessor,
    pub suggestion_line_processing: &'l dyn LineProcessor,
}

impl Default for CommanderView<'_> {
    fn default() -> Self {
        Self {
            input_top: false,
            input_line_processing: &no_processing,
            suggestion_line_processing: &no_processing,
        }
    }
}

#[inline]
fn no_processing(line: Line) -> Line {
    line
}

impl<'l> ratatui::widgets::StatefulWidget for CommanderView<'l> {
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

impl<'l> CommanderView<'l> {
    fn render_input(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut crate::Commander,
    ) {
        let line = self
            .input_line_processing
            .process(ratatui::text::Line::from(state.input()));
        line.render(area, buf);
    }

    fn render_suggestions(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut crate::Commander,
    ) {
        let suggestions = state
            .suggestions()
            .into_iter()
            .map(ratatui::widgets::ListItem::from)
            .collect::<Vec<_>>();

        let list = ratatui::widgets::List::new(suggestions)
            .highlight_symbol(">")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        ratatui::widgets::StatefulWidget::render(
            list,
            area,
            buf,
            state.suggestion_list_state_mut(),
        );
    }
}

pub trait LineProcessor {
    fn process<'l>(&self, line: Line<'l>) -> Line<'l>;
}

impl<F> LineProcessor for F
where
    F: Fn(Line<'_>) -> Line<'_>,
{
    fn process<'l>(&self, line: Line<'l>) -> Line<'l> {
        (self)(line)
    }
}
