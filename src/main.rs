extern crate sdl2;

use extend::ext;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::time::{Duration, Instant};

const LOGICAL_SCREEN_HEIGHT: u16 = 1080;
const LOGICAL_SCREEN_WIDTH: u16 = 1920;
const PHYSICAL_SCREEN_HEIGHT: u16 = 1080;
const PHYSICAL_SCREEN_WIDTH: u16 = 1920;

const DEGREES_IN_RADIANS: f64 = 0.0174533;
const FOV: f64 = 100.0;
use std::f64::consts::PI;
const DOF: f64 = 20.0;

const MOVE_SPEED: f64 = 0.03;
const TURN_SPEED: f64 = 0.05;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Wall {
    Empty,
    Wall,
}

struct Player {
    x: f64,
    y: f64,
    angle: f64,
}

#[derive(Clone, Copy)]
struct Vec2 {
    x: f64,
    y: f64,
}
impl Vec2 {
    fn get_length(&self) -> f64 {
        return self.x.hypot(self.y);
    }
}
impl std::ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl std::ops::Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
#[ext]
impl [[Wall; 10]; 10] {
    fn is_wall_at_position(&self, position: Vec2) -> bool {
        if !(position.x > 0.0 && position.x < 10.0 && position.y > 0.0 && position.y < 10.0) {
            return false;
        }
        let first_index: usize = position.y.trunc() as usize;
        let second_index: usize = position.x.trunc() as usize;
        //println!("index: {}, {}", first_index, second_index);
        match self[first_index][second_index] {
            Wall::Empty => return false,
            Wall::Wall => {
                println!("{} | {}", position.x, position.y);
                return true;
            }
        }
    }
}

struct RayCaster {
    x: f64,
    y: f64,
    angle: f64,
}
impl RayCaster {
    fn from_player(player: &Player) -> RayCaster {
        return RayCaster {
            x: player.x,
            y: player.y,
            angle: player.angle,
        };
    }
    fn cast(&self, map: [[Wall; 10]; 10]) -> f64 {
        //NaN needs to be handled
        let angle = angle_to_normal_range(self.angle);
        let sin: f64 = angle.sin();
        let cos: f64 = angle.cos();

        //println!("{} | {}, {}", angle, sin, cos);

        let step_total_delta = Vec2 {
            x: (1.0 / cos).abs(),
            y: (1.0 / sin).abs(),
        };
        let direction_coefficient = Vec2 {
            x: {
                if angle < PI * 1.5 && angle > PI * 0.5 {
                    -1.0
                } else {
                    1.0
                }
            },
            y: {
                if angle < PI && angle > 0.0 {
                    -1.0
                } else {
                    1.0
                }
            },
        };
        let mut delta_needed_for_next_tile = Vec2 {
            x: ((0.5 + 0.5 * direction_coefficient.x) - self.x.fract() * direction_coefficient.x)
                * step_total_delta.x,
            y: ((0.5 + 0.5 * direction_coefficient.y) - self.y.fract() * direction_coefficient.y)
                * step_total_delta.y,
        };
        let mut current_tile = Vec2 {
            x: (self.x + 0.0 * (direction_coefficient.x * 0.5 + 0.5)).trunc(),
            y: (self.y + 0.0 * (direction_coefficient.y * 0.5 + 0.5)).trunc(),
        };
        let mut travelled_distance = 0.0;

        loop {
            if delta_needed_for_next_tile.x < delta_needed_for_next_tile.y {
                delta_needed_for_next_tile.y -= delta_needed_for_next_tile.x;
                travelled_distance += delta_needed_for_next_tile.x.abs();
                delta_needed_for_next_tile.x = step_total_delta.x;
                current_tile.x += direction_coefficient.x;
            } else {
                delta_needed_for_next_tile.x -= delta_needed_for_next_tile.y;
                travelled_distance += delta_needed_for_next_tile.y.abs();
                delta_needed_for_next_tile.y = step_total_delta.y;
                current_tile.y += direction_coefficient.y;
            }

            if travelled_distance > DOF {
                return DOF;
            }
            let index_x: isize = current_tile.x as isize;
            let index_y: isize = current_tile.y as isize;
            if !(index_x < 0 || index_x > 9 || index_y < 0 || index_y > 9) {
                if map[index_x as usize][index_y as usize] == Wall::Wall {
                    return travelled_distance;
                }
            }
        }
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
        angle: 0.0 * DEGREES_IN_RADIANS,
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
    map[3][2] = Wall::Wall;
    map[3][1] = Wall::Wall;
    map[1][1] = Wall::Wall;
    map[6][7] = Wall::Wall;

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
            //distance *= (player.angle - ray_caster.angle).cos();
            let height: u16 = (LOGICAL_SCREEN_HEIGHT as f64 / distance).round() as u16;
            let brightness = (1.0 / (distance / 2.0).powi(2) * 255.0).round() as u8;
            canvas.set_draw_color(Color::RGB(brightness, brightness, brightness));

            canvas
                .draw_line(
                    Point::new(i as i32, (LOGICAL_SCREEN_HEIGHT / 2 + height / 2) as i32),
                    Point::new(i as i32, (LOGICAL_SCREEN_HEIGHT / 2 - height / 2) as i32),
                )
                .unwrap();

            ray_caster.angle =
                angle_to_normal_range(ray_caster.angle - delta_radians_per_iteration);
        }

        canvas.present();
        frames_delta_time = now.elapsed();
        println!("_______________________________________________________");
        println!("distance: {}", RayCaster::from_player(&player).cast(map));
        println!(
            "x:{}, y:{} | angle:{} | delta_time (micros): {}",
            player.x,
            player.y,
            player.angle,
            frames_delta_time.as_micros()
        );
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

#[test]
fn cast_test() {
    let mut map: [[Wall; 10]; 10] = [[Wall::Empty; 10]; 10];
    for i in 0..10 {
        for j in 0..10 {
            if j == 0 || i == 0 || j == 9 || i == 9 {
                map[i][j] = Wall::Wall;
            }
        }
    }
    let mut caster = RayCaster {
        x: 2.0,
        y: 6.0,
        angle: 0.0,
    };
    let distance = caster.cast(map);
    println!("{} | expected : 7.0", distance);
    assert!(distance == 7.0);
    caster.angle = PI * 0.5;
    let distance = caster.cast(map);
    println!("{} | expected : 5.0", distance);
    assert!(distance == 5.0);
    caster.angle = PI;
    let distance = caster.cast(map);
    println!("{} | expected : 1.0", distance);
    assert!(distance == 1.0);
    caster.angle = 1.5 * PI;
    let distance = caster.cast(map);
    println!("{} | expected : 3.0", distance);
    assert!(distance == 3.0);
    caster = RayCaster {
        x: 6.4,
        y: 3.7,
        angle: 0.0,
    };
    let distance = caster.cast(map);
    println!("{} | expected : 2.6", distance);
    //assert!(distance == 2.6);
    caster.angle = PI * 0.5;
    let distance = caster.cast(map);
    println!("{} | expected : 2.7", distance);
    //assert!(distance == 2.7);
    caster.angle = PI;
    let distance = caster.cast(map);
    println!("{} | expected : 5.4", distance);
    //assert!(distance == 5.4);
    caster.angle = 1.5 * PI;
    let distance = caster.cast(map);
    println!("{} | expected : 5.3", distance);
    //assert!(distance == 5.3);
    assert!(false);
}
