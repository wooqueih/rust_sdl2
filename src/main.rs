extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::time::{Duration, Instant};

const LOGICAL_SCREEN_HEIGHT: u16 = 9;
const LOGICAL_SCREEN_WIDTH: u16 = 16;
const PHYSICAL_SCREEN_HEIGHT: u16 = 1080;
const PHYSICAL_SCREEN_WIDTH: u16 = 1920;

const DEGREES_IN_RADIANS: f64 = 0.0174533;
const FOV: f64 = 90.0 * DEGREES_IN_RADIANS;
use std::f64::consts::PI;
const DOF: f64 = 20.0;

const MOVE_SPEED: f64 = 0.02;
const TURN_SPEED: f64 = 0.02;

#[derive(Clone, Copy)]
enum Wall {
    Empty,
    Wall,
}

struct Player {
    x: f64,
    y: f64,
    angle: f64,
}
struct Vec2 {
    x: f64,
    y: f64,
}
impl Vec2 {
    fn get_length(&self) -> f64 {
        return self.x.hypot(self.y);
    }
}

struct RayCaster {
    x: f64,
    y: f64,
    angle: f64,
    x_offset: f64,
    y_offset: f64,
}
impl RayCaster {
    fn from_player(player: &Player) -> RayCaster {
        return RayCaster {
            x: player.x,
            y: player.y,
            angle: player.angle,
            x_offset: player.x.fract().abs(),
            y_offset: player.y.fract().abs(),
        };
    }
    fn cast(&mut self, map: [[Wall; 10]; 10]) -> f64 {
        let mut cast_position = Vec2 {
            x: self.x,
            y: self.y,
        };

        let tan: f64 = self.angle.tan();
        let cot: f64 = 1.0 / tan;

        let mut vertical_total = Vec2 { x: 0.0, y: 0.0 };
        let mut direction_coefficient: f64;
        if self.angle < PI && self.angle > 0.0 {
            direction_coefficient = -1.0;
            vertical_total.y = self.y_offset;
            vertical_total.x = vertical_total.y * tan;
        } else {
            direction_coefficient = 1.0;
            vertical_total.y = self.y_offset;
            vertical_total.x = (1.0 - vertical_total.y) * tan;
        }
        'vertical_cast: loop {
            if cast_position.x > 0.0
                && cast_position.x < 10.0
                && cast_position.y > 0.0
                && cast_position.y < 10.0
            {
                let first_index: usize = cast_position.y.trunc() as usize;
                let second_index: usize = cast_position.x.trunc() as usize;
                //println!("index: {}, {}", first_index, second_index);
                match map[first_index][second_index] {
                    Wall::Empty => break,
                    Wall::Wall => {
                        println!("{} | {}", cast_position.x, cast_position.y);
                        break 'vertical_cast;
                    }
                }
            }
            if cast_position.x.is_nan()
                || cast_position.y.is_nan()
                || (cast_position).get_length() > DOF
            {
                vertical_total.y = DOF;
                vertical_total.x = 0.0;
                break;
            }

            cast_position.y += -1.0 * direction_coefficient;
            cast_position.x += tan;
        }
        let mut horizontal_total = Vec2 { x: 0.0, y: 0.0 };
        if self.angle < 1.5 * PI && self.angle > 0.0 {
            direction_coefficient = -1.0;
            horizontal_total.x = self.x_offset;
            horizontal_total.y = horizontal_total.x * cot;
        } else {
            direction_coefficient = 1.0;
            horizontal_total.x = self.x_offset;
            horizontal_total.y = (1.0 - horizontal_total.x) * cot;
        }

        /*
        'horizontal_cast: loop {
            if cast_position.x > 0.0 && cast_position.x < 10.0 && cast_position.y > 0.0 && cast_position.y < 10.0 {
                let first_index: usize = cast_position.y.trunc() as usize;
                let second_index: usize = cast_position.x.trunc() as usize;
                match map[first_index][second_index] {
                    Wall::Empty => break,
                    Wall::Wall => {
                        break 'horizontal_cast;
                    }
                }
            }
            if horizontal_total.x.is_nan()
                || horizontal_total.y.is_nan()
                || horizontal_total.get_length() > DOF
            {
                horizontal_total.y = DOF;
                horizontal_total.x = 0;
                break;
            }

            cast_position.y += -1*direction_coefficient;
            cast_position.x += tan;
        }*/
        horizontal_total.x = DOF;
        horizontal_total.y = 0.0;
        return vertical_total
            .get_length()
            .min(horizontal_total.get_length());
    }
}

fn main() {
    let sdl_context = sdl2::init().expect("sdl could not init");
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = sdl2::video::WindowBuilder::new(
        &video_subsystem,
        "test",
        PHYSICAL_SCREEN_WIDTH as u32,
        PHYSICAL_SCREEN_HEIGHT as u32,
    )
    .build()
    .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas
        .set_logical_size(LOGICAL_SCREEN_WIDTH as u32, LOGICAL_SCREEN_HEIGHT as u32)
        .unwrap();

    let mut key_w_down = false;
    let mut key_a_down = false;
    let mut key_s_down = false;
    let mut key_d_down = false;

    let mut player = Player {
        x: 5.0,
        y: 5.0,
        angle: 6.0,
    };

    let mut frames_delta_time: Duration = Duration::from_millis(10);
    let mut map = [[Wall::Empty; 10]; 10];
    for i in 0..10 {
        for j in 0..10 {
            if j == 0 || i == 0 || j == 9 || i == 9 {
                map[i][j] = Wall::Wall;
            }
        }
    }

    'game_loop: loop {
        let now = Instant::now();

        //input
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'game_loop,
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    key_w_down = true;
                    break;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    key_w_down = false;
                    break;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    key_a_down = true;
                    break;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    key_a_down = false;
                    break;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    key_s_down = true;
                    break;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    key_s_down = false;
                    break;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    key_d_down = true;
                    break;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    key_d_down = false;
                    break;
                }

                _ => {}
            }
        }
        let adjusted_move_speed = MOVE_SPEED * (frames_delta_time.as_micros() as f64) / 10000.0;
        let adjusted_turn_speed = TURN_SPEED * (frames_delta_time.as_micros() as f64) / 10000.0;
        if key_w_down {
            player.x += player.angle.cos() * adjusted_move_speed;
            player.y -= player.angle.sin() * adjusted_move_speed;
        }
        if key_s_down {
            player.x -= player.angle.cos() * adjusted_move_speed;
            player.y += player.angle.sin() * adjusted_move_speed;
        }
        if key_a_down {
            player.angle += adjusted_turn_speed;
        }
        if key_d_down {
            player.angle -= adjusted_turn_speed;
        }

        //rendering
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let mut ray_caster = RayCaster::from_player(&player);
        ray_caster.angle =
            angle_to_normal_range(ray_caster.angle + (FOV * DEGREES_IN_RADIANS) * 0.5);
        let delta_radians_per_iteration = (FOV * DEGREES_IN_RADIANS) / LOGICAL_SCREEN_WIDTH as f64;
        for i in 0..LOGICAL_SCREEN_WIDTH {
            let mut distance = ray_caster.cast(map);
            if distance < 1.0 {
                distance = 1.0;
            }
            let height: u16 = (LOGICAL_SCREEN_HEIGHT as f64 / distance).round() as u16;
            let brightness = (1.0 / (distance).powi(1) * 255.0).round() as u8;
            canvas.set_draw_color(Color::RGB(brightness, brightness, brightness));

            canvas
                .draw_line(
                    Point::new(i as i32, ((LOGICAL_SCREEN_HEIGHT + height) / 2) as i32),
                    Point::new(i as i32, ((LOGICAL_SCREEN_HEIGHT - height) / 2) as i32),
                )
                .unwrap();

            ray_caster.angle =
                angle_to_normal_range(ray_caster.angle - delta_radians_per_iteration);
        }

        canvas.present();
        frames_delta_time = now.elapsed();
        /*println!("distance: {}", RayCaster::from_player(&player).cast(map));
        println!(
            "x:{}, y:{} | angle:{} | delta_time (microseconds): {}",
            player.x,
            player.y,
            player.angle,
            frames_delta_time.as_micros()
        );*/
    }
}
fn angle_to_normal_range(input_angle: f64) -> f64 {
    let mut angle = input_angle;
    loop {
        if angle > 2.0 * PI {
            angle -= 2.0 * PI;
        } else if angle < 0.0 {
            angle += 2.0 * PI;
        } else {
            break;
        }
    }
    return angle;
}
