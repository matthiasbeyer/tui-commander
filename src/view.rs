use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::text::Line;
use ratatui::widgets::ListItem;
use ratatui::widgets::Widget;

#[derive(Default)]
pub struct CommanderView {
    input_top: bool,
    suggestion_highlight_symbol: Option<String>,
    suggestion_highlight_spacing: Option<ratatui::widgets::HighlightSpacing>,
    input_line_processing: Option<Box<dyn Fn(Line<'_>) -> Line<'_>>>,
    suggestion_line_processing: Option<Box<dyn Fn(usize, ListItem<'_>) -> ListItem<'_>>>,
}

impl CommanderView {
    pub fn with_input_line_processing<P>(mut self, proc: P) -> Self
    where
        P: Fn(Line<'_>) -> Line<'_>,
        P: 'static,
    {
        self.input_line_processing = Some(Box::new(proc));
        self
    }

    pub fn with_suggestion_line_processing<P>(mut self, proc: P) -> Self
    where
        P: Fn(usize, ListItem<'_>) -> ListItem<'_>,
        P: 'static,
    {
        self.suggestion_line_processing = Some(Box::new(proc));
        self
    }

    pub fn with_suggestion_highlight_spacing(
        mut self,
        suggestion_highlight_spacing: Option<ratatui::widgets::HighlightSpacing>,
    ) -> Self {
        self.suggestion_highlight_spacing = suggestion_highlight_spacing;
        self
    }

    pub fn with_suggestion_highlight_symbol(
        mut self,
        suggestion_highlight_symbol: Option<String>,
    ) -> Self {
        self.suggestion_highlight_symbol = suggestion_highlight_symbol;
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
        let suggestions = state
            .suggestions()
            .into_iter()
            .map(ratatui::widgets::ListItem::from)
            .enumerate()
            .map(|(i, list_item)| {
                if let Some(proc) = self.suggestion_line_processing.as_ref() {
                    (proc)(i, list_item)
                } else {
                    list_item
                }
            })
            .collect::<Vec<_>>();

        let list = ratatui::widgets::List::new(suggestions);
        let list = if let Some(sym) = self.suggestion_highlight_symbol.as_ref() {
            list.highlight_symbol(sym)
        } else {
            list
        };
        let list = if let Some(space) = self.suggestion_highlight_spacing.as_ref() {
            list.highlight_spacing(space.clone())
        } else {
            list
        };

        ratatui::widgets::StatefulWidget::render(
            list,
            area,
            buf,
            state.suggestion_list_state_mut(),
        );
    }
}
