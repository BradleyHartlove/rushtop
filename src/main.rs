use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::Gauge,
};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

pub struct App {
    exit: bool,
    system: System,
    cpu_usages: Vec<f32>,
}

impl Default for App {
    fn default() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );
        // CPU usage is diff-based; sleep once at startup to get a real first reading.
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();
        let cpu_usages = system.cpus().iter().map(|c| c.cpu_usage()).collect();
        Self { exit: false, system, cpu_usages }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            self.system.refresh_cpu_usage();
            self.cpu_usages = self.system.cpus().iter().map(|c| c.cpu_usage()).collect();
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let n = self.cpu_usages.len();
        if n == 0 {
            return;
        }
        let rows = Layout::vertical((0..n).map(|_| Constraint::Length(1)))
            .split(frame.area());

        for (i, (usage, row)) in self.cpu_usages.iter().zip(rows.iter()).enumerate() {
            let label = format!("CPU{:2} {:5.1}%", i, usage);
            frame.render_widget(
                Gauge::default()
                    .gauge_style(Style::default().fg(Color::Green))
                    .ratio((*usage as f64 / 100.0).clamp(0.0, 1.0))
                    .label(label),
                *row,
            );
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.code == KeyCode::Char('q') {
            self.exit = true;
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key_event(key);
                }
            }
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
