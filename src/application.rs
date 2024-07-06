use eframe::egui::*;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use crate::audio::AudioHandler;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 224;
const SCALE: usize = 2;
const FRAME_BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
const ROM_SIZE: usize = 8_192;

pub struct App {
    frame_buffer: Arc<Mutex<Box<Vec<Color32>>>>,
    device1: Arc<Mutex<u8>>,
    device2: Arc<Mutex<u8>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            frame_buffer: Arc::new(Mutex::new(Box::new(vec![Color32::BLACK; FRAME_BUFFER_SIZE * SCALE * SCALE]))),
            device1: Arc::new(Mutex::new(0)),
            device2: Arc::new(Mutex::new(0)),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = App::default();
        let ctx_clone = cc.egui_ctx.clone();
        let frame_buffer_clone = app.frame_buffer.clone();
        let device1 = app.device1.clone();
        let device2 = app.device2.clone();

        std::thread::spawn(move || {
            let mut rom = [0; ROM_SIZE];
            let rom_paths: [&str; 4] = ["invaders.h", "invaders.g", "invaders.f", "invaders.e"];
            for i in 0..4 {
                let data = std::fs::read(rom_paths[i]).unwrap();
                for (pos, e) in data.iter().enumerate() {
                    rom[(i * 2048) + pos] = *e;
                }
            }
            let memory = Box::new(intel8080::memory::Memory::new(rom));
            let mut c = intel8080::emulator::Cpu::new(memory);
            println!("8080 CPU started");

            let mut shift_register: u16 = 0;
            let mut shift_register_offest: u8 = 0;

            let mut audio = AudioHandler::new();
            let mut last_device3: u8 = 0b00000000;
            let mut last_device5: u8 = 0b00000000;
            let mut start = Instant::now();
            loop {
                let mut tick = 0;
                while tick < 33333 {
                    if tick == 16667 {
                        c.receive_interrupt(0xCF);
                    }
                    c.tick();
                    match c.get_output() {
                        Some(x) => {
                            let (device, value) = x;
                            match device {
                                0x2 => {
                                    shift_register_offest = value & 0x07;
                                },
                                0x3 => {
                                    if value & 0b00000001 == 0b00000001 && last_device3 & 0b00000001 != 0b00000001{
                                        audio.play_sound(0);
                                    }
                                    if value & 0b00000010 == 0b00000010 && last_device3 & 0b00000010 != 0b00000010 {
                                        audio.play_sound(1);
                                    }
                                    if value & 0b00000100 == 0b00000100 && last_device3 & 0b00000100 != 0b00000100 {
                                        audio.play_sound(2);
                                    }
                                    if value & 0b00001000 == 0b00001000 && last_device3 & 0b00001000 != 0b00001000 {
                                        audio.play_sound(3);
                                    }
                                    last_device3 = value;
                                },
                                0x4 => {
                                    shift_register = ((value as u16) << 8) | (shift_register >> 8);
                                },
                                0x5 => {
                                    if value & 0b00000001 == 0b00000001 && last_device5 & 0b00000001 != 0b00000001 {
                                        audio.play_sound(4);
                                    }
                                    if value & 0b00000010 == 0b00000010 && last_device5 & 0b00000010 != 0b00000010 {
                                        audio.play_sound(5);
                                    }
                                    if value & 0b00000100 == 0b00000100 && last_device5 & 0b00000100 != 0b00000100 {
                                        audio.play_sound(6);
                                    }
                                    if value & 0b00001000 == 0b00001000 && last_device5 & 0b00001000 != 0b00001000 {
                                        audio.play_sound(7);
                                    }
                                    if value & 0b00010000 == 0b00010000 && last_device5 & 0b00010000 != 0b00010000 {
                                        audio.play_sound(8);
                                    }
                                    last_device5 = value;
                                },
                                0x6 => {}, //OUT 6  Watchdog not implemented.
                                _ => panic!("Invalid OUT device number.")
                            }
                        },
                        None => {}
                    }
                    c.set_input(0, 0b10001111);
                    c.set_input(1, device1.lock().unwrap().clone());
                    c.set_input(2, device2.lock().unwrap().clone());
                    c.set_input(3, (shift_register >> (8 - shift_register_offest)) as u8);
                    tick += 1;
                }
                c.receive_interrupt(0xD7);
                
                let vram = c.get_vram();
                let mut rows: Vec<Vec<Color32>> = vec![];
                let mut current_row: Vec<Color32> = vec![];
                for index in 0..7_168 {
                    for offset in 0..8 {
                        let val = vram[index] >> offset & 0x1;
                        if val == 1 {
                            for _ in 0..SCALE {
                                current_row.push(Color32::WHITE);
                            }
                        } else {
                            for _ in 0..SCALE {
                                current_row.push(Color32::BLACK);
                            }
                        }
                    }
                    if current_row.len() == SCREEN_WIDTH * SCALE {
                        for _ in 0..SCALE {
                            rows.push(current_row.clone());
                        }
                        current_row = vec![];
                    }
                }

                let time_spent = start.elapsed().as_micros();
                if time_spent < 16667 as u128 {
                    thread::sleep(Duration::from_micros(16667 - time_spent as u64))
                }
                *frame_buffer_clone.lock().unwrap() = Box::new(rows.concat());
                ctx_clone.request_repaint();
                start = Instant::now();
            }
        });
        return app;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(25.0);
            let image = ColorImage { size: [SCREEN_WIDTH * SCALE, SCREEN_HEIGHT * SCALE], pixels: *self.frame_buffer.lock().unwrap().clone(), };
            let texture = ctx.load_texture("display", image, TextureOptions::LINEAR);
            let rotated_image = egui::Image::from_texture(&texture).rotate(-1.5708, Vec2::splat(0.5));
            ui.add(rotated_image);
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            let mut device1_bits = 0b00001000;
            let mut device2_bits = 0b00000000;
            if ctx.input(|i| i.key_pressed(Key::Space)) {
                device1_bits = device1_bits | 0b00000001;
            }
            if ctx.input(|i| i.key_pressed(Key::Num1)) {
                device1_bits = device1_bits | 0b00000100;
            }
            if ctx.input(|i| i.key_pressed(Key::Num2)) {
                device1_bits = device1_bits | 0b00000010;
            }
            if ctx.input(|i| i.key_pressed(Key::W)) {
                device1_bits = device1_bits | 0b00010000;
            }
            if ctx.input(|i| i.key_pressed(Key::A)) {
                device1_bits = device1_bits | 0b00100000;
            }
            if ctx.input(|i| i.key_pressed(Key::D)) {
                device1_bits = device1_bits | 0b01000000;
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
                device2_bits = device2_bits | 0b00100000;
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
                device2_bits = device2_bits | 0b01000000;
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
                device2_bits = device2_bits | 0b00010000;
            }
            *self.device1.lock().unwrap() = device1_bits;
            *self.device2.lock().unwrap() = device2_bits;
        });
    }
}