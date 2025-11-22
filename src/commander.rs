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
    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    #[inline]
    pub fn reset_input(&mut self) {
        self.input = String::new();
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

    pub(crate) fn input(&self) -> &str {
        &self.input
    }

    pub(crate) fn suggestion_list_state_mut(&mut self) -> &mut ratatui::widgets::ListState {
        &mut self.suggestion_list_state
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
