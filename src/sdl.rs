extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::rc::Rc;
use std::time::Duration;

pub struct SDL {
    context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    window: Rc<sdl2::video::Window>,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

impl SDL {
    pub fn new() -> SDL {
        let context = sdl2::init().unwrap();
        let video_subsystem = context.video().unwrap();
        let window = Rc::new(
            video_subsystem
                .window("rust-sdl2 demo", 800, 600)
                .position_centered()
                .build()
                .unwrap(),
        );

        let clone_window = Rc::clone(&window);
        let canvas = clone_window().into_canvas().build().unwrap();

        Self {
            context,
            video_subsystem,
            window,
            canvas,
        }
    }

    pub fn run(&mut self) {
        let mut event_pump = self.context.event_pump().unwrap();
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
            self.canvas.set_draw_color(Color::RGB(0, 0, 0));
            self.canvas.clear();
            self.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
