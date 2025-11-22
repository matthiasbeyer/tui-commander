pub trait Command: 'static {
    const NAME: &'static str;
    type Error: std::error::Error + 'static;
    type Args: 'static;

    /// This function gets called repeatedly as long as the user types
    ///
    /// It should return an error if the args are not valid for the command, which then is logged
    /// and the input is highlighted
    fn parse_args(&self, args: Vec<String>) -> Result<Self::Args, Self::Error>;

    /// As soon as the user hits <CR> / Enter / Return, this function gets called with the
    /// Ok(Self::Args) from the Command::parse_args() function
    fn run(&self, args: Self::Args) -> Result<(), Self::Error>;
}

pub(crate) struct ErasedCommand {
    object: Box<dyn std::any::Any>,
    fn_parse_args: fn(&dyn std::any::Any, Vec<String>) -> Result<Args, Error>,
    fn_run: fn(&dyn std::any::Any, Args) -> Result<(), Error>,
}

pub(crate) struct Error(pub(crate) Box<dyn std::error::Error>);
pub(crate) struct Args(Box<dyn std::any::Any>);

impl ErasedCommand {
    pub(crate) fn erase<C>(command: C) -> Self
        where C: Command,
    {
        Self {
            fn_parse_args: |object, args| -> Result<Args, Error> {
                let command: &C = (*object).downcast_ref().unwrap();
                match command.parse_args(args) {
                    Ok(args) => Ok(Args(Box::new(args))),
                    Err(error) => Err(Error(Box::new(error))),
                }
            },

            fn_run: |object, Args(args)| -> Result<(), Error> {
                let command: &C = object.downcast_ref().unwrap();
                let args: C::Args = *args.downcast().unwrap();
                match command.run(args) {
                    Ok(()) => Ok(()),
                    Err(error) => Err(Error(Box::new(error))),
                }
            },

            object: Box::new(command),
        }
    }

    pub(crate) fn parse_args(&self, args: Vec<String>) -> Result<Args, Error> {
        (self.fn_parse_args)(&self.object, args)
    }

    pub(crate) fn run(&self, args: Args) -> Result<(), Error> {
        (self.fn_run)(&self.object, args)
    }
}



