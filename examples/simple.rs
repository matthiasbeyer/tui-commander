use color_eyre::Result;
use crossterm::event::Event;
use crossterm::event::EventStream;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use futures::FutureExt;
use futures::StreamExt;
use ratatui::widgets::Widget;
use ratatui::DefaultTerminal;
use tui_commander::Command;
use tui_commander::Commander;
use tui_commander::Context;

#[derive(Debug)]
pub struct CommandContext {
    continue_running: bool,
    lines: Vec<String>,
}

impl Context for CommandContext {}

#[derive(Debug, thiserror::Error)]
#[error("Some error happend")]
pub struct Error;

static_assertions::assert_impl_all!(Error: Send, Sync);

pub struct QuitCommand;

impl Command<CommandContext> for QuitCommand {
    fn name() -> &'static str {
        "quit"
    }

    fn args_are_valid(args: &[&str]) -> bool {
        args.is_empty()
    }

    fn build_from_command_name_str(
        input: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        if input == "quit" || input == "qui" || input == "qu" || input == "q" {
            Ok(QuitCommand)
        } else {
            Err(Box::new(Error))
        }
    }

    fn execute(
        &self,
        arguments: Vec<String>,
        context: &mut CommandContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        if !arguments.is_empty() {
            // eprintln!("Arguments to 'quit' command... I don't take any");
        }

        context.continue_running = false;
        Ok(())
    }
}

pub struct EchoCommand;

impl Command<CommandContext> for EchoCommand {
    fn name() -> &'static str {
        "echo"
    }

    fn args_are_valid(_: &[&str]) -> bool {
        true
    }

    fn build_from_command_name_str(
        input: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        if input == "echo" || input == "ech" || input == "ec" || input == "e" {
            Ok(EchoCommand)
        } else {
            Err(Box::new(Error))
        }
    }

    fn execute(
        &self,
        arguments: Vec<String>,
        context: &mut CommandContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        context.lines.push(arguments.join(" "));
        Ok(())
    }
}

pub struct PopCommand;

impl Command<CommandContext> for PopCommand {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "pop"
    }

    fn build_from_command_name_str(
        input: &str,
    ) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>>
    where
        Self: Sized,
    {
        if input == "pop" || input == "po" || input == "p" {
            Ok(PopCommand)
        } else {
            Err(Box::new(Error))
        }
    }

    fn args_are_valid(_: &[&str]) -> bool
    where
        Self: Sized,
    {
        false
    }

    fn execute(
        &self,
        _arguments: Vec<String>,
        context: &mut CommandContext,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        context.lines.pop();
        Ok(())
    }
}

async fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut commander = Commander::builder()
        .with_case_sensitive(false)
        .with_command::<QuitCommand>()
        .with_command::<EchoCommand>()
        .with_command::<PopCommand>()
        .build();

    let mut context = CommandContext {
        continue_running: true,
        lines: Vec::new(),
    };

    let mut command_ui = tui_commander::ui::Ui::default();
    let mut commander_active = false;
    let mut events = EventStream::new();
    loop {
        terminal.draw(|frame| {
            ratatui::layout::Layout::vertical(
                context
                    .lines
                    .iter()
                    .map(|_| ratatui::layout::Constraint::Min(3)),
            )
            .split(frame.area())
            .iter()
            .zip(context.lines.iter())
            .for_each(|(area, line)| {
                ratatui::text::Line::from(line.clone()).render(*area, frame.buffer_mut())
            });

            if commander_active {
                frame.render_stateful_widget(&mut command_ui, frame.area(), &mut commander);
            }
        })?;

        match events.next().fuse().await.unwrap().unwrap() {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(':') if !commander_active => {
                            commander_active = true;
                        }

                        KeyCode::Enter if commander_active => {
                            match commander.execute(&mut context) {
                                Err(e) => {
                                    println!("Error: {e:?}");
                                }
                                Ok(_) => {
                                    commander_active = false;
                                    command_ui.reset();
                                }
                            }
                        }

                        KeyCode::Esc if commander_active => {
                            commander_active = false;
                            command_ui.reset();
                        }

                        _ => {
                            if commander_active {
                                command_ui.handle_key_press(key);
                                commander.set_input(command_ui.value().to_string());
                            } else {
                                // do nothing for now
                            }
                        }
                    }
                }
            }

            _other_event => {
                // do nothing for now
            }
        }

        if !context.continue_running {
            break Ok(());
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal).await;
    ratatui::restore();
    result
}
