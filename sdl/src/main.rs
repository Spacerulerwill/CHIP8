use chip8::{Chip8, SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, video::Window,
};
use std::{
    env,
    fs::File,
    io::Read,
    time::{Duration, Instant},
};

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

const FPS: u32 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS as u64);
const INSTRUCTION_PER_FRAME: u32 = 10;

pub fn main() {
    // File name
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("No filename found");
    }

    // Setup SDL
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    // Setup emulator, reading file
    let mut chip8 = Chip8::new();
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    // Game loop
    let mut event_pump = sdl_context.event_pump().unwrap();

    'gameloop: loop {
        let frame_start = Instant::now(); // Mark the start of the frame

        // Handle events
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(key) = get_key_button(key) {
                        chip8.keypress(key, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(key) = get_key_button(key) {
                        chip8.keypress(key, false);
                    }
                }
                _ => (),
            }
        }

        // Run emulator cycles - execute more ticks based on calculated ticks_to_run
        for _ in 0..INSTRUCTION_PER_FRAME {
            chip8.tick();
        }
        chip8.tick_timers();

        // Draw the screen
        draw_screen(&mut chip8, &mut canvas);

        // Calculate how long the frame took
        let frame_duration = frame_start.elapsed();

        // If the frame took less time than the target frame duration, sleep for the remaining time
        if frame_duration < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - frame_duration);
        }
    }
}

fn draw_screen(chip: &mut Chip8, canvas: &mut Canvas<Window>) {
    // Clear the canvas with black color
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = chip.get_display();
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    // Draw each pixel from the CHIP-8 screen buffer
    for (i, &pixel) in screen_buf.iter().enumerate() {
        if pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }

    // Present the canvas to the screen
    canvas.present();
}

fn get_key_button(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
