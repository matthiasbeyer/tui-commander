use std::collections::HashMap;

use crate::Command;

pub struct Commander {
    input: String,
    commands: HashMap<&'static str, crate::command::ErasedCommand>,
    suggestion_list_state: ratatui::widgets::ListState,
}

impl Commander {
    pub fn builder() -> CommanderBuilder {
        CommanderBuilder {
            commands: HashMap::new(),
        }
    }

    #[inline]
    pub fn push_to_input(&mut self, chr: char) {
        self.input.push(chr);
    }

    #[inline]
    pub fn backspace(&mut self) {
        let _ = self.input.pop();
    }

    #[inline]
    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    #[inline]
    pub fn reset_input(&mut self) {
        self.input = String::new();
    }

    #[inline]
    pub fn next_suggestion(&mut self) {
        self.suggestion_list_state.select_next();
    }

    #[inline]
    pub fn prev_suggestion(&mut self) {
        self.suggestion_list_state.select_previous();
    }

    // TODO: Matching algorithm: Currently prefix, but fuzzy would be nice
    pub fn suggestions(&self) -> Vec<String> {
        let Some(first_word) = self.input.split_whitespace().next() else {
            return Vec::new();
        };

        self.commands
            .keys()
            .filter(|name| name.starts_with(first_word))
            .map(ToString::to_string)
            .collect()
    }

    pub fn use_selected_suggestion(&mut self) {
        if let Some(sel) = self.suggestion_list_state.selected() {
            let mut input_split = self.input.split_whitespace();
            let Some(first_word) = input_split.next() else {
                return;
            };

            let Some((name, _cmd)) = self
                .commands
                .iter()
                .filter(|(name, _cmd)| name.starts_with(first_word))
                .nth(sel)
            else {
                return;
            };

            let mut new_input = vec![*name];
            new_input.extend(input_split);

            self.set_input(new_input.join(" "));
        }
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn suggestion_list_state_mut(&mut self) -> &mut ratatui::widgets::ListState {
        &mut self.suggestion_list_state
    }

    pub fn suggestion_list_state(&self) -> &ratatui::widgets::ListState {
        &self.suggestion_list_state
    }

    /// Run the current command
    ///
    /// Runs either what is currently selected from the suggestion list, if anything is selected,
    /// or the command that matches the first word
    ///
    /// Returns Ok(None) if no command was run, Ok(Some(())) if the command ran successfully or
    /// Err(_) if the command errored
    pub fn run(&mut self) -> Result<Option<()>, Box<dyn std::error::Error>> {
        let mut input_split = self.input.split_whitespace();
        let Some(first_word) = input_split.next() else {
            return Ok(None);
        };

        let input_rest = input_split.map(ToString::to_string).collect::<Vec<_>>();

        let mut all_relevant_commands = self
            .commands
            .iter()
            .filter(|(name, _cmd)| name.starts_with(first_word));

        let command = if let Some(sel) = self.suggestion_list_state.selected() {
            all_relevant_commands
                .nth(sel)
                .map(|(_name, command)| command)
        } else {
            all_relevant_commands.next().map(|(_name, cmd)| cmd)
        };

        if let Some(command) = command {
            let args = match command.parse_args(input_rest) {
                Ok(args) => args,
                Err(error) => return Err(error.0),
            };

            let res = command.run(args).map_err(|e| e.0).map(Some);
            self.reset_input();
            res
        } else {
            Ok(None)
        }
    }
}

pub struct CommanderBuilder {
    commands: HashMap<&'static str, crate::command::ErasedCommand>,
}

impl CommanderBuilder {
    pub fn with_command<C>(mut self, command: C) -> Self
    where
        C: Command,
    {
        self.commands
            .insert(C::NAME, crate::command::ErasedCommand::erase(command));
        self
    }

    pub fn build(mut self) -> Commander {
        Commander {
            input: String::new(),
            suggestion_list_state: ratatui::widgets::ListState::default(),
            commands: {
                self.commands.shrink_to_fit();
                self.commands
            },
        }
    }
}
