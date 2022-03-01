use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::fs;
use std::io::Read;
use std::time::Duration;

use clap::Parser;

use chip8::Chip8;

pub const SQUARE_SIZE: usize = 16;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Rom to open
    #[clap(short, long)]
    rom: Option<String>,

    /// Instructions per frame
    #[clap(long, default_value_t = 10)]
    ipf: usize,
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn get_rom(path: &str) -> Vec<u8> {
    let mut rom = vec![];
    fs::OpenOptions::new()
        .read(true)
        .open(path)
        .expect("rom not found")
        .read_to_end(&mut rom)
        .expect("couldn't read rom");

    rom
}

fn main() {
    // Parse arguments
    let args = Args::parse();
    let ipf = args.ipf;

    let mut chip = Chip8::new();

    // initialize SDL stuff
    let sdl_context = sdl2::init().expect("couldn't initialize SDL");
    let video_subsystem = sdl_context
        .video()
        .expect("couldn't initialize the video subsystem");
    let audio_subsystem = sdl_context
        .audio()
        .expect("couldn't initialize the audio subsystem");

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let sound = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.1,
            }
        })
        .expect("couldn't open audio device");

    let window = video_subsystem
        .window(
            "Rusty Chip",
            (SQUARE_SIZE * SCREEN_WIDTH) as u32,
            (SQUARE_SIZE * SCREEN_HEIGHT) as u32,
        )
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("could not make a canvas");
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().expect("event pump error");

    // Open and load rom
    let path = if let Some(path) = args.rom {
        path
    } else {
        let path;
        'selecting: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => return,
                    Event::DropFile { filename, .. } => {
                        path = filename;
                        break 'selecting;
                    }
                    _ => {}
                }

                if let Event::DropFile { filename, .. } = event {
                    path = filename;
                    break 'selecting;
                }
            }
        }
        path
    };
    let rom = get_rom(&path);
    chip.load_rom(&rom).expect("couldn't load rom");

    let mut pause = false;
    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Escape => return,
                    Keycode::P => pause = !pause,
                    Keycode::Num1 => chip.key_down(0x1),
                    Keycode::Num2 => chip.key_down(0x2),
                    Keycode::Num3 => chip.key_down(0x3),
                    Keycode::Num4 => chip.key_down(0xc),
                    Keycode::Q => chip.key_down(0x4),
                    Keycode::W => chip.key_down(0x5),
                    Keycode::E => chip.key_down(0x6),
                    Keycode::R => chip.key_down(0xd),
                    Keycode::A => chip.key_down(0x7),
                    Keycode::S => chip.key_down(0x8),
                    Keycode::D => chip.key_down(0x9),
                    Keycode::F => chip.key_down(0xe),
                    Keycode::Z => chip.key_down(0xa),
                    Keycode::X => chip.key_down(0x0),
                    Keycode::C => chip.key_down(0xb),
                    Keycode::V => chip.key_down(0xf),
                    _ => {}
                },

                Event::KeyUp {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Num1 => chip.key_up(0x1),
                    Keycode::Num2 => chip.key_up(0x2),
                    Keycode::Num3 => chip.key_up(0x3),
                    Keycode::Num4 => chip.key_up(0xc),
                    Keycode::Q => chip.key_up(0x4),
                    Keycode::W => chip.key_up(0x5),
                    Keycode::E => chip.key_up(0x6),
                    Keycode::R => chip.key_up(0xd),
                    Keycode::A => chip.key_up(0x7),
                    Keycode::S => chip.key_up(0x8),
                    Keycode::D => chip.key_up(0x9),
                    Keycode::F => chip.key_up(0xe),
                    Keycode::Z => chip.key_up(0xa),
                    Keycode::X => chip.key_up(0x0),
                    Keycode::C => chip.key_up(0xb),
                    Keycode::V => chip.key_up(0xf),
                    _ => {}
                },
                Event::DropFile { filename, .. } => {
                    let rom = get_rom(&filename);
                    chip.reset();
                    chip.load_rom(&rom).expect("couldn't load rom");
                }

                _ => {}
            }
        }

        // Go to the next frame if the game is not paused
        if !pause {
            chip.frame(ipf).expect("emulation error");
        }

        // Audio update
        if chip.buzzer() {
            sound.resume();
        } else {
            sound.pause();
        }

        // Video update
        let fb = chip.fb();
        for (y, row) in fb.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                if *pixel {
                    canvas.set_draw_color(Color::WHITE);
                } else {
                    canvas.set_draw_color(Color::BLACK);
                }
                canvas
                    .fill_rect(Rect::new(
                        (x * SQUARE_SIZE) as i32,
                        (y * SQUARE_SIZE) as i32,
                        SQUARE_SIZE as u32,
                        SQUARE_SIZE as u32,
                    ))
                    .expect("failed to draw a rect");
            }
        }
        canvas.present();

        // Wait for 15ms
        std::thread::sleep(Duration::from_millis(15));
    }
}
