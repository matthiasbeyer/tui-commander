pub trait Command<Context> {
    /// The name of the command, what a user has to type to find the command and execute
    fn name() -> &'static str
    where
        Self: Sized;

    fn build_from_command_name_str(
        input: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>>
    where
        Self: Sized;

    fn args_are_valid(args: &[&str]) -> bool
    where
        Self: Sized;

    fn execute(
        &self,
        arguments: Vec<String>,
        context: &mut Context,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;
}

pub struct CommandBox<Context>(pub(crate) Box<dyn Command<Context>>);
