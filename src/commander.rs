use std::collections::HashMap;
use std::ops::Deref;

use crate::command::CommandBox;
use crate::Command;

pub struct Commander<Context> {
    command_builders: HashMap<&'static str, CommandFuncs<Context>>,

    command_str: String,

    search_engine: nucleo_matcher::Matcher,
    active: bool,
}

impl<Context> Commander<Context>
where
    Context: crate::Context,
{
    pub fn builder() -> CommanderBuilder<Context> {
        CommanderBuilder {
            case_sensitive: false,
            command_builders: HashMap::new(),
        }
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.active
    }

    #[inline]
    pub fn start(&mut self) {
        self.active = true
    }

    #[inline]
    pub fn reset(&mut self) {
        self.active = false
    }

    pub fn suggestions(&mut self) -> Vec<String> {
        let all_commands = self.all_command_names();

        if let Some((command, _args)) = self.get_command_args() {
            let command = command.to_string();
            self.suggestions_for_command(command, all_commands)
                .collect::<Vec<String>>()
        } else {
            all_commands
        }
    }

    pub fn is_unknown_command(&mut self) -> bool {
        self.suggestions().is_empty()
    }

    pub fn set_input(&mut self, input: String) {
        self.command_str = input;
    }

    pub fn execute(&mut self, context: &mut Context) -> Result<(), CommanderError> {
        let Some((command, args)) = self.get_currently_matching_command_and_args() else {
            return Err(CommanderError::EmptyCommand);
        };

        let Some(command_funcs) = self.command_builders.get(command.deref()) else {
            return Err(CommanderError::UnknownCommand(self.command_str.clone()));
        };

        let commandbox = (command_funcs.builder)(&command)?;
        commandbox
            .0
            .execute(args, context)
            .map_err(CommanderError::Command)
    }

    fn all_command_names(&self) -> Vec<String> {
        self.command_names()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
    }

    fn suggestions_for_command(
        &mut self,
        command: String,
        all_commands: Vec<String>,
    ) -> impl Iterator<Item = String> {
        nucleo_matcher::pattern::Pattern::new(
            command.as_ref(),
            nucleo_matcher::pattern::CaseMatching::Ignore,
            nucleo_matcher::pattern::Normalization::Never,
            nucleo_matcher::pattern::AtomKind::Fuzzy,
        )
        .match_list(all_commands, &mut self.search_engine)
        .into_iter()
        .map(|tpl| tpl.0)
        .map(|s| s.to_string())
    }

    fn command_names(&self) -> impl Iterator<Item = &str> {
        self.command_builders.keys().copied()
    }

    fn find_command_funcs_for_command(
        &self,
        command: &str,
    ) -> Result<&CommandFuncs<Context>, CommanderError> {
        self.command_builders
            .get(command)
            .ok_or_else(|| CommanderError::UnknownCommand(self.command_str.clone()))
    }

    fn get_command_args(&self) -> Option<(&str, Vec<&str>)> {
        let mut it = self.command_str.split(' ');
        let command = it.next()?;
        let args = it.collect();
        Some((command, args))
    }

    fn get_currently_matching_command_and_args(&mut self) -> Option<(String, Vec<String>)> {
        let (command, args) = self.get_command_args()?;
        let args = args.into_iter().map(String::from).collect();

        self.suggestions_for_command(command.to_string(), self.all_command_names())
            .next()
            .map(|command| (command, args))
    }

    pub(crate) fn current_args_are_valid(&self) -> Result<bool, CommanderError> {
        let Some((command, args)) = self.get_command_args() else {
            return Err(CommanderError::EmptyCommand);
        };

        let funcs = self.find_command_funcs_for_command(command)?;
        Ok((funcs.arg_validator)(&args))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommanderError {
    #[error("Command execution errored")]
    Command(Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("Empty command string")]
    EmptyCommand,

    #[error("Unknown command {}", .0)]
    UnknownCommand(String),
}

struct CommandFuncs<Context> {
    builder: CommandBuilderFn<Context>,
    arg_validator: CommandArgValidatorFn,
}

type CommandBuilderFn<Context> = Box<dyn Fn(&str) -> Result<CommandBox<Context>, CommanderError>>;
type CommandArgValidatorFn = Box<dyn Fn(&[&str]) -> bool>;

pub struct CommanderBuilder<Context> {
    case_sensitive: bool,
    command_builders: HashMap<&'static str, CommandFuncs<Context>>,
}

impl<Context> CommanderBuilder<Context> {
    pub fn with_case_sensitive(mut self, b: bool) -> Self {
        self.case_sensitive = b;
        self
    }

    pub fn with_command<C>(mut self) -> Self
    where
        C: Command<Context> + Send + Sync + 'static,
        Context: 'static,
    {
        fn command_builder<C, Context>(input: &str) -> Result<CommandBox<Context>, CommanderError>
        where
            C: Command<Context> + Send + Sync + 'static,
            Context: 'static,
        {
            C::build_from_command_name_str(input)
                .map(|c| CommandBox(Box::new(c) as Box<dyn Command<Context>>))
                .map_err(CommanderError::Command)
        }

        fn arg_validator<C, Context>(args: &[&str]) -> bool
        where
            C: Command<Context> + Send + Sync + 'static,
            Context: 'static,
        {
            C::args_are_valid(args)
        }

        self.command_builders.insert(
            C::name(),
            CommandFuncs {
                builder: Box::new(command_builder::<C, Context>),
                arg_validator: Box::new(arg_validator::<C, Context>),
            },
        );
        self
    }

    pub fn build(mut self) -> Commander<Context> {
        self.command_builders.shrink_to_fit();
        let search_engine = nucleo_matcher::Matcher::new({
            let mut config = nucleo_matcher::Config::DEFAULT;
            config.ignore_case = !self.case_sensitive;
            config.prefer_prefix = true;
            config
        });

        Commander {
            command_builders: self.command_builders,

            active: false,
            search_engine,
            command_str: String::new(),
        }
    }
}
