use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::widgets::{Block, Paragraph, Widget};
use std::time::{Duration};
use std::{env, thread};
use crossterm::event::{self, KeyCode};
use eyre::{Context, Result}; // Or std::result::Result with your error type
struct Particle {
    x: u16,
    y: u16,
    ch: char,
}
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        min + (self.next() as usize % (max - min))
    }
}

struct MR<'a> {
    particles: &'a [Particle],
}

impl<'a> Widget for MR<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for p in self.particles {
            if p.x < area.width && p.y < area.height {
                buf[(area.x + p.x, area.y + p.y)]
                    .set_char(p.ch)
                    .set_fg(Color::DarkGray);
            }
        }
    }
}
fn should_quit() -> Result<bool, Box<dyn std::error::Error>> {
    if event::poll(Duration::from_millis(5)).context("event poll failed")? {
        let q_pressed = event::read()
            .context("event read failed")?
            .as_key_press_event()
            .is_some_and(|key| key.code == KeyCode::Char('q'));
        return Ok(q_pressed);
    }
    Ok(false)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let has_s = args.iter().any(|arg| arg == "-s");
    let has_o = args.iter().any(|arg| arg == "-o");
    let mut message = "SERN";
    if has_o && !has_s {
        message = "Organization";
    }

    ratatui::run(|terminal| -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = SimpleRng::new(1337);
        let mut particles = Vec::new();
        let initial_area = terminal.size()?;

        // Başlangıç parçacıkları (harfler)
        for _ in 0..20 {
            particles.push(Particle {
                x: rng.gen_range(0, initial_area.width.max(1) as usize) as u16,
                y: rng.gen_range(0, initial_area.height.max(1) as usize) as u16,
                ch: (rng.gen_range(33, 126) as u8) as char,
            });
        }

        // 5 saniye boyunca akıcı bir animasyon döngüsü
        loop {
            let size = terminal.size()?;
            // Parçacık konumlarını güncelle (aşağı kayma ve rastgele yer değiştirme)
            for p in &mut particles {
                p.y = (p.y + 1) % size.height.max(1);
                if rng.gen_range(0, 10) > 7 {
                    p.x = rng.gen_range(0, size.width.max(1) as usize) as u16;
                    p.ch = (rng.gen_range(33, 126) as u8) as char;
                }
            }
            terminal.draw(|frame| {
                let area = frame.area();

                // 1. Önce arka plana uçuşan harfleri çiziyoruz
                let block = Block::bordered()
                    .title("El Psy Kongroo")
                    .border_style(Color::Cyan); // Çerçeve rengi (Örn: Cyan)

                // Yazı rengini .yellow() ile ayarlıyoruz
                let greeting = Paragraph::new(format!("{} is Listening", message))
                    .yellow()
                    .centered();
                let timepassed = Paragraph::new(format!("0.{}{}{}{}", rng.gen_range(0,9), rng.gen_range(0,9), rng.gen_range(0,9), rng.gen_range(0,9)))
                    .blue()
                    .centered();

                // Çerçevenin iç alanını alıp dikey olarak ortalıyoruz
                let inner_area = block.inner(frame.area());
                let vl = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Ratio(1, 2),
                        Constraint::Length(2),
                        Constraint::Ratio(1, 2),
                    ])
                    .split(inner_area);
                let text_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Length(1)])
                    .split(vl[1]);
                frame.render_widget(greeting, text_layout[0]);
                frame.render_widget(timepassed, text_layout[1]);
                let rain = MR {
                    particles: &particles,
                };
                frame.render_widget(rain, area);

                frame.render_widget(block.clone(), frame.area());
            })?;
            if should_quit()? {
                break;
            }
            thread::sleep(Duration::from_millis(80));
        }
        Ok(())
    })?;

    Ok(())
}