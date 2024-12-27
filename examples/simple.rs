use color_eyre::Result;
use crossterm::event::Event;
use crossterm::event::EventStream;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use futures::FutureExt;
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tui_commander::Command;
use tui_commander::Commander;
use tui_commander::Context;

#[derive(Debug)]
pub struct CommandContext {
    continue_running: bool,
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

    fn args_are_valid(_: &[&str]) -> bool {
        false
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
        _arguments: Vec<String>,
        _context: &mut CommandContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(())
    }
}

async fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut commander = Commander::builder()
        .with_case_sensitive(false)
        .with_command::<QuitCommand>()
        .with_command::<EchoCommand>()
        .build();

    let mut context = CommandContext {
        continue_running: true,
    };

    let mut command_ui = tui_commander::ui::Ui::default();

    let mut events = EventStream::new();
    loop {
        terminal.draw(|frame| {
            frame.render_stateful_widget(&mut command_ui, frame.area(), &mut commander);
        })?;

        match events.next().fuse().await.unwrap().unwrap() {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(':') if !commander.is_active() => {
                            commander.start();
                        }

                        KeyCode::Enter if commander.is_active() => {
                            if let Err(e) = commander.execute(&mut context) {
                                println!("Error: {:?}", e);
                            }
                        }

                        KeyCode::Esc if commander.is_active() => {
                            commander.reset();
                        }

                        _ => {
                            if commander.is_active() {
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
