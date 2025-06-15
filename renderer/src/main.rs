use minifb::{Key, Window, WindowOptions};
use std::cmp::{max, min};

const WIDTH: usize = 600;
const HEIGHT: usize = 600;

// Colors so I don't go insane type hexcodes
const BLACK: u32 = 0x000000;
const WHITE: u32 = 0xFFFFFF;
const RED:   u32 = 0xFF0000;
const GREEN: u32 = 0x00FF00;
const BLUE:  u32 = 0x0000FF;
const YELLOW:u32 = 0xFFFF00;

fn reset_screen() -> Vec<u32> {
    return vec![0; WIDTH * HEIGHT];
}

fn make_line(buf: &mut [u32], p1_x: usize, p1_y: usize, p2_x: usize, p2_y: usize, color: u32) {
    let x0 = p1_x as isize;
    let y0 = p1_y as isize;
    let x1 = p2_x as isize;
    let y1 = p2_y as isize;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();

    let steps = dx.max(dy);
    if steps == 0 {return;}

    let x_step = (x1 - x0) as f32 / steps as f32;
    let y_step = (y1 - y0) as f32 / steps as f32;

    let mut x = x0 as f32;
    let mut y = y0 as f32;

    for _ in 0..steps + 1 {
        let px = x.round() as isize;
        let py = y.round() as isize;

        if px >= 0 && px < WIDTH as isize && py >= 0 && py < HEIGHT as isize {
            let idx = py as usize * WIDTH + px as usize;
            buf[idx] = color;
        }

        x += x_step;
        y += y_step;
    }
}

fn make_square_filled(buf: &mut [u32], cx: usize, cy: usize, size: i32, color: u32){
    for y in -size / 2..size / 2 {
        for x in -size / 2..size / 2 {
            let px = cx as isize + x as isize;
            let py = cy as isize + y as isize;
            if px >= 0 && px < WIDTH as isize && py >= 0 && py < HEIGHT as isize {
                let idx = py as usize * WIDTH + px as usize;
                buf[idx] = color;
            }
        }
    }
}

fn make_square(buf: &mut [u32], cx: usize, cy: usize, size: i32, color: u32){
    let c1_x = cx - (size / 2) as usize; 
    let c1_y = cy - (size / 2) as usize;
    let c2_x = cx + (size / 2) as usize;
    let c2_y = cy + (size / 2) as usize;

    make_line(buf, c1_x, c1_y, c1_x, c2_y, color); 
    make_line(buf, c1_x, c1_y, c2_x, c1_y, color); 
    make_line(buf, c2_x, c2_y, c1_x, c2_y, color); 
    make_line(buf, c2_x, c2_y, c2_x, c1_y, color); 
}

fn make_triangle(buf: &mut [u32], v1_x: usize, v1_y: usize, v2_x: usize, v2_y: usize, v3_x: usize, v3_y: usize, color: u32) {
    make_line(buf, v1_x, v1_y, v3_x, v3_y, color); 
    make_line(buf, v2_x, v2_y, v3_x, v3_y, color);
    make_line(buf, v2_x, v2_y, v1_x, v1_y, color);
}


fn main() {
    let mut buffer: Vec<u32> = reset_screen(); 

    let mut window = Window::new("Baby Steps", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| panic!("Error making a window! You goofed!: {}", e));

    while window.is_open() && !window.is_key_down(Key::Q) {
        buffer = reset_screen(); 
        // make_square(&mut buffer, 300, 300, 100, GREEN);
        // make_square(&mut buffer, 300, 300, 80, GREEN);

        // make_square(&mut buffer, 300, 300, 80, GREEN); 
        make_triangle(&mut buffer, 100, 500, 350, 200, 500, 500, GREEN);
        make_triangle(&mut buffer, 150, 550, 400, 250, 550, 550, RED); 
        
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}