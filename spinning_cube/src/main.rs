use minifb::{Key, Window, WindowOptions};

// https://github.com/saatvikrao/Spinning-Cube/blob/main/spinning_cube.c

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const FAR: f64 = 40.0; 

const CUBE_WIDTH: i32 = 20; 

const RED:   u32 = 0xFF0000;
const GREEN: u32 = 0x00FF00;
const BLUE:  u32 = 0x0000FF;



fn reset_screen() -> Vec<u32> {
    return vec![0; WIDTH * HEIGHT];
}

fn update_side(buf: &mut [u32], zbuf: &mut [f64], cX: f64, cY: f64, cZ: f64, ax: f64, ay: f64, az: f64, hof: f64, color: u32) {
    let coords = get_coords(cX, cY, cZ, ax, ay, az); 
    let x = coords.0; let y = coords.1; let z = coords.2 + FAR; 
    let ooz = 1.0 / z; 

    let scale = 100.0; 
    let xp = ((WIDTH / 2) as f64 + hof + scale * ooz * x) as i32;
    let yp = ((HEIGHT / 2) as f64 + scale * ooz * y) as i32; 

    let idx = xp + yp * (WIDTH as i32);
    
    if idx >= 0 && idx < (WIDTH * HEIGHT) as i32 {
        if ooz > zbuf[idx as usize] {
            zbuf[idx as usize] = ooz;
            buf[idx as usize] = color;
        }
    }
}


fn get_coords(i: f64, j: f64, k: f64, ax: f64, ay: f64, az: f64) -> (f64, f64, f64) {
    let (mut x, mut y, mut z) = (i, j * ax.cos() - k * ax.sin(), j * ax.sin() + k * ax.cos());
    let (nx, ny, nz) = (x * ay.cos() + z * ay.sin(), y, -x * ay.sin() + z * ay.cos());
    let (fx, fy, fz) = (nx * az.cos() - ny * az.sin(), nx * az.sin() + ny * az.cos(), nz);

    (fx, fy, fz)
}

fn main() {
    let mut buffer: Vec<u32> = reset_screen(); 

    let mut window = Window::new("Spinning Cube", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| panic!("Error making a window! You goofed!: {}", e));
    
    let mut ax = 0.0;
    let mut ay = 0.0;
    let mut az = 0.0;
    let hof = -2.0 * (CUBE_WIDTH as f64); 
    let is = 0.4; 

    while window.is_open() && !window.is_key_down(Key::Q) {
        buffer = reset_screen(); 
        let mut zbuf = vec![f64::NEG_INFINITY; WIDTH * HEIGHT]; 

        let mut cx = -1.0 * CUBE_WIDTH as f64;
        
        while cx < CUBE_WIDTH as f64 {
            let mut cy = -1.0 * CUBE_WIDTH as f64;
            while cy < CUBE_WIDTH as f64 {
                update_side(&mut buffer, &mut zbuf, cx, cy, -1.0 * CUBE_WIDTH as f64, ax, ay, az, hof, RED); 
                update_side(&mut buffer, &mut zbuf, CUBE_WIDTH as f64, cy, cx , ax, ay, az, hof, GREEN); 
                update_side(&mut buffer, &mut zbuf, -1.0 * CUBE_WIDTH as f64, cy, -1.0 * cx, ax, ay, az, hof, GREEN); 
                update_side(&mut buffer, &mut zbuf, -1.0 * cx, cy, CUBE_WIDTH as f64, ax, ay, az, hof, RED); 

                update_side(&mut buffer, &mut zbuf, cx, -1.0 * CUBE_WIDTH as f64, -1.0 * cy, ax, ay, az, hof, BLUE); 
                update_side(&mut buffer, &mut zbuf, cx, CUBE_WIDTH as f64, cy, ax, ay, az, hof, BLUE); 
                cy += is; 
            }
            cx += is; 
        } 

        ax += 0.01;
        ay += 0.01;
        az += 0.01; 

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}