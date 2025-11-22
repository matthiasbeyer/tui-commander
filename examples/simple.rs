use color_eyre::Result;
use crossterm::event::EventStream;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use futures::FutureExt;
use futures::StreamExt;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Widget;
use ratatui::DefaultTerminal;
use tui_commander::Command;
use tui_commander::Commander;

#[derive(Debug, thiserror::Error)]
#[error("Some error happend")]
pub struct Error;

static_assertions::assert_impl_all!(Error: Send, Sync);

pub enum AppEvent {
    Quit,
    Echo(String),
}

type EventSender = tokio::sync::mpsc::Sender<AppEvent>;

macro_rules! cmd {
    ($tname:ident => $name:literal, args: $args:ty, parse: $parse:expr, run: $run:expr) => {
        pub struct $tname {
            sender: EventSender,
        }

        impl Command for $tname {
            const NAME: &'static str = $name;
            type Error = Error;
            type Args = $args;

            fn parse_args(
                &self,
                args: Vec<String>,
            ) -> std::result::Result<Self::Args, Self::Error> {
                $parse(args)
            }

            fn run(&self, args: Self::Args) -> std::result::Result<(), Self::Error> {
                $run(self, args)
            }
        }
    };
}

cmd! {
    QuitCommand => "quit",
    args: (),
    parse: |_args| Ok(()),
    run: |this: &QuitCommand, _args: ()| {
        let sender = this.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender.send(AppEvent::Quit).await {
                eprintln!("{error:?}");
            }
        });
        Ok(())
    }
}

cmd! {
    EchoCommand => "echo",
    args: String,
    parse: |args: Vec<String>| Ok(args.join(", ")),
    run: |this: &EchoCommand, args: String| {
        let sender = this.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender.send(AppEvent::Echo(args)).await {
                eprintln!("{error:?}");
            }
        });
        Ok(())
    }
}

cmd! {
    EchoSeperateCommand => "echo-seperate",
    args: Vec<String>,
    parse: |args: Vec<String>| Ok(args),
    run: |this: &EchoSeperateCommand, args: Vec<String>| {
        let sender = this.sender.clone();
        tokio::spawn(async move {
            for arg in args {
                if let Err(error) = sender.send(AppEvent::Echo(arg)).await {
                    eprintln!("{error:?}");
                }
            }
        });
        Ok(())
    }
}

async fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let (app_event_sender, mut app_event_recv) = tokio::sync::mpsc::channel(10);
    let mut commander = Commander::builder()
        .with_command(QuitCommand {
            sender: app_event_sender.clone(),
        })
        .with_command(EchoCommand {
            sender: app_event_sender.clone(),
        })
        .with_command(EchoSeperateCommand {
            sender: app_event_sender.clone(),
        })
        .build();

    let mut commander_active = false;
    let mut echoed_lines: Vec<String> = vec![];
    let mut events = EventStream::new();
    loop {
        terminal.draw(|frame| {
            let [log_area, commander_area] =
                Layout::vertical(vec![Constraint::Fill(2), Constraint::Fill(1)])
                    .areas(frame.area());

            ratatui::widgets::Paragraph::new(
                echoed_lines
                    .iter()
                    .map(ToString::to_string)
                    .map(ratatui::text::Line::from)
                    .collect::<Vec<_>>(),
            )
            .block(ratatui::widgets::Block::bordered().title("Echoed"))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .render(log_area, frame.buffer_mut());

            if commander_active {
                let block = ratatui::widgets::Block::bordered();
                let inner_commander_area = block.inner(commander_area);
                block.render(commander_area, frame.buffer_mut());

                let line = ratatui::text::Line::from(commander.input());
                let suggestions = {
                    let list = commander
                        .suggestions()
                        .into_iter()
                        .enumerate()
                        .map(|(i, line)| {
                            ratatui::widgets::ListItem::from(line).style(Style::default().fg(
                                if i % 2 == 0 {
                                    Color::Blue
                                } else {
                                    Color::Green
                                },
                            ))
                        });

                    ratatui::widgets::List::new(list)
                        .highlight_symbol(">> ")
                        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
                        .block(ratatui::widgets::Block::bordered())
                };

                let [suggestions_area, input_area] =
                    Layout::vertical(vec![Constraint::Percentage(100), Constraint::Min(1)])
                        .areas(inner_commander_area);

                line.render(input_area, frame.buffer_mut());
                ratatui::widgets::StatefulWidget::render(
                    suggestions,
                    suggestions_area,
                    frame.buffer_mut(),
                    commander.suggestion_list_state_mut(),
                );
            }
        })?;

        match events.next().fuse().await.unwrap().unwrap() {
            crossterm::event::Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(':') if !commander_active => {
                            commander_active = true;
                        }
                        KeyCode::Up if commander_active => commander.prev_suggestion(),
                        KeyCode::Down if commander_active => commander.next_suggestion(),

                        KeyCode::Enter if commander_active => {
                            match commander.run() {
                                Ok(Some(())) => {}
                                Ok(None) => {}
                                Err(error) => {
                                    eprintln!("Error running commander: {error:?}");
                                }
                            }

                            commander_active = false;
                        }

                        KeyCode::Esc if commander_active => {
                            commander_active = false;
                            commander.reset_input();
                        }

                        _ => {
                            if commander_active {
                                match key.code {
                                    KeyCode::Char(chr) => commander.push_to_input(chr),
                                    KeyCode::Backspace => commander.backspace(),
                                    _ => { // ignored
                                    }
                                }
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

        if let Ok(Some(event)) =
            tokio::time::timeout(std::time::Duration::from_millis(10), app_event_recv.recv()).await
        {
            match event {
                AppEvent::Quit => break,
                AppEvent::Echo(args) => {
                    echoed_lines.push(args);
                }
            }
        }
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal).await;
    ratatui::restore();
    result
}
