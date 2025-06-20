use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const FOV: f64 = 90.0f64.to_radians();
const ASPECT_RATIO: f64 = WIDTH as f64 / HEIGHT as f64; 

// Colors so I don't go insane type hexcodes
const BLACK: u32 = 0x000000;
const WHITE: u32 = 0xFFFFFF;
const RED:   u32 = 0xFF0000;
const GREEN: u32 = 0x00FF00;
const BLUE:  u32 = 0x0000FF;
const YELLOW:u32 = 0xFFFF00;


// some 3D structs, I don't know if doing it this way is smart
// but I will probably put this in a different file later anyway
// everything is a triangle so it doesn't matter if this is it lol
#[derive(Clone, Copy)]
struct V3 {x: f64, y: f64, z: f64}
struct Triangle3d {v0: V3, v1: V3, v2: V3, color: u32}

fn reset_screen() -> Vec<u32> {
    return vec![0; WIDTH * HEIGHT];
}

// project 3D coordinates to 2D coordinates, this is probably
// the backbone of all 3D stuff here
fn project_3D_to_2D(mut v: V3) -> (i32, i32) {
    v.z += 250.0;
    if v.z <= 0.01 {
        return (WIDTH as i32 / 2, HEIGHT as i32 / 2); // fallback
    }

    let scale = 1.0 / (FOV * 0.5).tan();
    let aspect_ratio = WIDTH as f64 / HEIGHT as f64;

    let x = (v.x * scale / (v.z * aspect_ratio)) * WIDTH as f64 / 2.0 + WIDTH as f64 / 2.0;
    let y = -(v.y * scale / v.z) * HEIGHT as f64 / 2.0 + HEIGHT as f64 / 2.0;

    (x as i32, y as i32)
}

// fairly standard way to make a line, just step through
fn make_line(buf: &mut [u32], p1_x: usize, p1_y: usize, p2_x: usize, p2_y: usize, color: u32) {
    let x0 = p1_x as isize;
    let y0 = p1_y as isize;
    let x1 = p2_x as isize;
    let y1 = p2_y as isize;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();

    let steps = dx.max(dy);
    if steps == 0 {return;}

    let x_step = (x1 - x0) as f64 / steps as f64;
    let y_step = (y1 - y0) as f64 / steps as f64;

    let mut x = x0 as f64;
    let mut y = y0 as f64;

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

// another potential way to do this is to just make a lot
// of lines but this is easier
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

// basic square with 4 lines
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

// another basic shape for 2D
fn make_triangle_2D(buf: &mut [u32], v1_x: usize, v1_y: usize, v2_x: usize, v2_y: usize, v3_x: usize, v3_y: usize, color: u32) {
    make_line(buf, v1_x, v1_y, v3_x, v3_y, color); 
    make_line(buf, v2_x, v2_y, v3_x, v3_y, color);
    make_line(buf, v2_x, v2_y, v1_x, v1_y, color);
}

// the other backbone of all 3D, the best primitive
fn make_triangle_3D(buf: &mut [u32], triangle: Triangle3d) {
    let (x0, y0) = project_3D_to_2D(triangle.v0);
    let (x1, y1) = project_3D_to_2D(triangle.v1);
    let (x2, y2) = project_3D_to_2D(triangle.v2);

    make_triangle_2D(
        buf,
        x0 as usize, y0 as usize,
        x1 as usize, y1 as usize,
        x2 as usize, y2 as usize,
        triangle.color,
    );
}

// unused right now, might not need it at all lol
fn rotate_y(v: V3, an: f64) -> V3 {
    let sin_a = an.sin();
    let cos_a = an.cos();
    V3 {
        x: v.x * cos_a - v.z * sin_a,
        y: v.y,
        z: v.x * sin_a + v.z * cos_a,
    }
}

// helper to make a Triangle struct
fn get_triangle_from_vecs(v0: V3, v1: V3, v2: V3, color: u32) -> Triangle3d {
    return Triangle3d {
        v0: v0, 
        v1: v1, 
        v2: v2,
        color: color, 
    }; 
}

// Unlike the 2D shapes that I just draw straight to the buffer, my idea
// with the 3D shapes is to create generators that output a list of triangles
// that can be then projected one at a time onto the buffer
// this will make doing manipulations like rotations, transforms, and translations easier
// ...once I get there
fn get_cube_triangles(size: i32, cx: usize, cy: usize, cz: usize, color: u32) -> Vec<Triangle3d> {
    let c1_x = cx - (size ) as usize; 
    let c1_y = cy - (size ) as usize;
    let c1_z = cz - (size ) as usize;

    let c2_x = cx + (size ) as usize; 
    let c2_y = cy + (size ) as usize;
    let c2_z = cz + (size ) as usize;

    // binary iteration
    // It's like the Klein-4 group but with three switches
    let v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    let v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    let v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    let v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    let v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    let v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    let v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    let v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    return vec![
    // triangles to make: 
        get_triangle_from_vecs(v_000, v_001, v_010, color), // 000-001-010
        get_triangle_from_vecs(v_000, v_001, v_100, color), // 000-001-100
        get_triangle_from_vecs(v_000, v_010, v_100, color), // 000-010-100
        
        get_triangle_from_vecs(v_111, v_101, v_110, color), // 111-101-110
        get_triangle_from_vecs(v_111, v_101, v_011, color), // 111-101-011
        get_triangle_from_vecs(v_111, v_110, v_011, color), // 111-110-011

        get_triangle_from_vecs(v_100, v_101, v_001, color), // 100-101-001
        get_triangle_from_vecs(v_100, v_101, v_110, color), // 100-101-110
        get_triangle_from_vecs(v_100, v_110, v_010, color), // 100-110-010

        get_triangle_from_vecs(v_011, v_101, v_001, color), // 011-101-001
        get_triangle_from_vecs(v_011, v_010, v_110, color), // 011-010-110
        get_triangle_from_vecs(v_011, v_001, v_010, color), // 011-001-010

    ];
}

// takes a list of triangles and adds them to the buffer, 
// Three.js does something like this with world.add()
fn draw_3d_from_triangles(buf: &mut [u32], triangles: Vec<Triangle3d>) {
    for triangle in triangles {
        make_triangle_3D(buf, triangle); 
    }
}

// this probably works but my cube rendering implementation is wrong right now
// so I can't verify it lol; I'll get back to this at some point
// unused and I'm probably going to remove it
fn rotate_y_triangle(triangles: Vec<Triangle3d>, angle: f64) -> Vec<Triangle3d> {
    let sin_a = angle.sin();
    let cos_a = angle.cos();

    let mut out_vec: Vec<Triangle3d> = vec![];
    
    for triangle in triangles {
        let vecs = vec![triangle.v0, triangle.v1, triangle.v2]; 
        let mut new_vecs: Vec<V3> = vec![];  
        for v in vecs {
            new_vecs.push(rotate_y(v, angle));  
        }
        out_vec.push(get_triangle_from_vecs(new_vecs[0], new_vecs[1], new_vecs[2], triangle.color)); 
    } 
    return out_vec; 
    
}

// rotate a list triangles, think of each call of this as using one rotation
// matrix and order matters as matrix mult is not commutative
fn rotate_triangles(mut triangles: Vec<Triangle3d>, ax: f64, ay: f64, az: f64) -> Vec<Triangle3d> {
    let mut out_vec: Vec<Triangle3d> = vec![];
    
    for mut triangle in &mut triangles {
        let v0 = rotate_point(triangle.v0.x, triangle.v0.y, triangle.v0.z, ax, ay, az);
        let v1 = rotate_point(triangle.v1.x, triangle.v1.y, triangle.v1.z, ax, ay, az);
        let v2 = rotate_point(triangle.v2.x, triangle.v2.y, triangle.v2.z, ax, ay, az); 

        triangle.v0.x = v0.0; triangle.v0.y = v0.1; triangle.v0.z = v0.2;
        triangle.v1.x = v1.0; triangle.v1.y = v1.1; triangle.v1.z = v1.2;
        triangle.v2.x = v2.0; triangle.v2.y = v2.1; triangle.v2.z = v2.2; 

    } 
    return triangles; 
    
}

// helper to rotate a specifc point
fn rotate_point(i: f64, j: f64, k: f64, ax: f64, ay: f64, az: f64) -> (f64, f64, f64) {
    let (mut x, mut y, mut z) = (i, j * ax.cos() - k * ax.sin(), j * ax.sin() + k * ax.cos());
    let (nx, ny, nz) = (x * ay.cos() + z * ay.sin(), y, -x * ay.sin() + z * ay.cos());
    let (fx, fy, fz) = (nx * az.cos() - ny * az.sin(), nx * az.sin() + ny * az.cos(), nz);

    (fx, fy, fz)
}


fn main() {
    let mut buffer: Vec<u32> = reset_screen(); 

    let mut window = Window::new("Baby Steps", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| panic!("Error making a window! You goofed!: {}", e));
    
    let mut angle = 0.0;
    while window.is_open() && !window.is_key_down(Key::Q) {
        buffer = reset_screen(); 
        // make_square(&mut buffer, 300, 300, 100, GREEN);
        // make_square(&mut buffer, 300, 300, 80, GREEN);

        // make_square(&mut buffer, 300, 300, 150, YELLOW); 
        // make_triangle_2D(&mut buffer, 100, 500, 350, 200, 500, 500, GREEN);
        // make_triangle_2D(&mut buffer, 150 - 100, 550 - 100, 400 - 100, 250 - 100, 550 - 100, 550 -100, RED); 
        
        // let cube1 = get_cube_triangles(35, 50, 50, 50, GREEN); 
        // draw_3d_from_triangles(&mut buffer, cube1);

        // let cube2 = get_cube_triangles(35, 50, 35, 50, RED); 
        // draw_3d_from_triangles(&mut buffer, cube2); 

        let cube3 = rotate_triangles(get_cube_triangles(50, 50, 50, 50, BLUE), angle, -1.0 * angle, -0.5 * angle);
        draw_3d_from_triangles(&mut buffer, cube3);
        angle += 0.01; 
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}