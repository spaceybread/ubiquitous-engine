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
#[derive(Clone, Copy)]
struct Triangle3d {v0: V3, v1: V3, v2: V3, color: u32}

fn reset_screen() -> Vec<u32> {
    return vec![0; WIDTH * HEIGHT];
}

// project 3D coordinates to 2D coordinates, this is probably
// the backbone of all 3D stuff here
fn project_3D_to_2D(mut v: V3) -> (i32, i32) {
    // Move camera backwards instead of pushing objects forward
    let camera_z = -150.0;
    v.z -= camera_z;

    if v.z <= 0.01 {
        return (WIDTH as i32 / 2, HEIGHT as i32 / 2); // clipped behind camera
    }

    let scale = 1.0 / (FOV * 0.5).tan();
    let aspect_ratio = WIDTH as f64 / HEIGHT as f64;

    let x_ndc = (v.x * scale) / (v.z * aspect_ratio); // normalized -1..1
    let y_ndc = -(v.y * scale) / v.z;

    let x_screen = (x_ndc * WIDTH as f64 / 2.0) + WIDTH as f64 / 2.0;
    let y_screen = (y_ndc * HEIGHT as f64 / 2.0) + HEIGHT as f64 / 2.0;

    (x_screen as i32, y_screen as i32)
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
    // fill_triangle(buf, x0, y0, x1, y1, x2, y2, triangle.color);
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

// helper to get an edge
fn edge_function(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64) -> f64 {
    (cx - ax) * (by - ay) - (cy - ay) * (bx - ax)
}

// making filled triangles
// dont think real life is just wireframes lol
fn fill_triangle(buf: &mut [u32], x0: i32, y0: i32, 
                 x1: i32, y1: i32, 
                 x2: i32, y2: i32, color: u32) {
    let min_x = x0.min(x1.min(x2)).max(0);
    let max_x = x0.max(x1.max(x2)).min(WIDTH as i32 - 1);
    let min_y = y0.min(y1.min(y2)).max(0);
    let max_y = y0.max(y1.max(y2)).min(HEIGHT as i32 - 1);

    let area = edge_function(x0 as f64, y0 as f64,
        x1 as f64, y1 as f64,
        x2 as f64, y2 as f64);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let w0 = edge_function(x1 as f64, y1 as f64,
                        x2 as f64, y2 as f64,
                        x as f64, y as f64);
            let w1 = edge_function(x2 as f64, y2 as f64,
                        x0 as f64, y0 as f64,
                        x as f64, y as f64);
            let w2 = edge_function(x0 as f64, y0 as f64,
                        x1 as f64, y1 as f64,
                        x as f64, y as f64);

            if (area >= 0.0 && w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0)
            || (area < 0.0 && w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
            let idx = y as usize * WIDTH + x as usize;
            buf[idx] = color;
            }
        }
    }
}

// time to add lighting

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

// wireframe hand! 
fn wireframe_hand(color1: u32) -> (Vec<Triangle3d>, Vec<(String, usize, usize)>) {
    let mut out_vec = vec![]; 
    let mut idx_tup = vec![]; 

    // base of the hand
    let mut size = 20.0;

    let mut cx = 0.0; 
    let mut cy = -30.0; 
    let mut cz = -15.0;

    let mut c1_x = cx - size;
    let mut c1_y = cy - size / 4.0;
    let mut c1_z = cz - size;

    let mut c2_x = cx + size;
    let mut c2_y = cy + size / 4.0;
    let mut c2_z = cz + size;

    // binary iteration
    // It's like the Klein-4 group but with three switches
    let mut v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    let mut v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    let mut v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    let mut v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    let mut v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    let mut v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    let mut v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    let mut v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    let mut color = RED; 
    let mut base_vecs = vec![
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

    let mut old_idx = out_vec.len(); 
    out_vec.append(&mut base_vecs);
    idx_tup.push(("base".to_string(), old_idx, out_vec.len())); 


    // wrist
    size = 10.0;

    cx = 0.0; 
    cy = -20.0; 
    cz = -15.0;

    c1_x = cx - size;
    c1_y = cy - size / 2.0;
    c1_z = cz - size;

    c2_x = cx + size;
    c2_y = cy + size / 2.0;
    c2_z = cz + size;

    v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    color = YELLOW;
    let mut wrist_vecs = vec![
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

    let mut old_idx = out_vec.len(); 
    out_vec.append(&mut wrist_vecs); 
    idx_tup.push(("wrist".to_string(), old_idx, out_vec.len()));

    // finger_1_top
    size = 8.0;

    cx = -20.0; 
    cy = 75.0; 
    cz = 10.0;

    c1_x = cx - size;
    c1_y = cy - size * 1.7;
    c1_z = cz - size;

    c2_x = cx + size;
    c2_y = cy + size * 1.7;
    c2_z = cz + size;

    v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    color = RED;
    let mut finger_vecs = vec![
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

    let mut old_idx = out_vec.len();
    out_vec.append(&mut finger_vecs); 
    idx_tup.push(("finger_1_top".to_string(), old_idx, out_vec.len()));

    // finger_1_bot
    size = 8.0;

    cx = -20.0; 
    cy = 45.0; 
    cz = 10.0;

    c1_x = cx - size;
    c1_y = cy - size * 1.7;
    c1_z = cz - size;

    c2_x = cx + size;
    c2_y = cy + size * 1.7;
    c2_z = cz + size;

    v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    color = YELLOW;
    let mut finger_vecs = vec![
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

    let mut old_idx = out_vec.len();
    out_vec.append(&mut finger_vecs);
    idx_tup.push(("finger_1_bot".to_string(), old_idx, out_vec.len()));

    // finger_2_top
    size = 8.0;

    cx = 0.0; 
    cy = 75.0; 
    cz = 10.0;

    c1_x = cx - size;
    c1_y = cy - size * 1.7;
    c1_z = cz - size;

    c2_x = cx + size;
    c2_y = cy + size * 1.7;
    c2_z = cz + size;

    v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    color = RED;
    let mut finger_vecs = vec![
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

    let mut old_idx = out_vec.len();
    out_vec.append(&mut finger_vecs); 
    idx_tup.push(("finger_2_top".to_string(), old_idx, out_vec.len()));

    // finger_2_bot
    size = 8.0;

    cx = 0.0; 
    cy = 45.0; 
    cz = 10.0;

    c1_x = cx - size;
    c1_y = cy - size * 1.7;
    c1_z = cz - size;

    c2_x = cx + size;
    c2_y = cy + size * 1.7;
    c2_z = cz + size;

    v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    color = YELLOW;
    let mut finger_vecs = vec![
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

    let mut old_idx = out_vec.len();
    out_vec.append(&mut finger_vecs); 
    idx_tup.push(("finger_2_bot".to_string(), old_idx, out_vec.len()));

    // finger_3_top
    size = 8.0;

    cx = 20.0; 
    cy = 75.0; 
    cz = 10.0;

    c1_x = cx - size;
    c1_y = cy - size * 1.7;
    c1_z = cz - size;

    c2_x = cx + size;
    c2_y = cy + size * 1.7;
    c2_z = cz + size;

    v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    color = RED;
    let mut finger_vecs = vec![
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

    let mut old_idx = out_vec.len();
    out_vec.append(&mut finger_vecs); 
    idx_tup.push(("finger_3_top".to_string(), old_idx, out_vec.len()));

    // finger_3_bot
    size = 8.0;

    cx = 20.0; 
    cy = 45.0; 
    cz = 10.0;

    c1_x = cx - size;
    c1_y = cy - size * 1.7;
    c1_z = cz - size;

    c2_x = cx + size;
    c2_y = cy + size * 1.7;
    c2_z = cz + size;

    v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    color = YELLOW;
    let mut finger_vecs = vec![
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

    let mut old_idx = out_vec.len();
    out_vec.append(&mut finger_vecs); 
    idx_tup.push(("finger_3_bot".to_string(), old_idx, out_vec.len()));

    // finger_4_top
    size = 8.0;

    cx = 40.0; 
    cy = 20.0; 
    cz = 10.0;

    c1_x = cx - size;
    c1_y = cy - size;
    c1_z = cz - size;

    c2_x = cx + size;
    c2_y = cy + size;
    c2_z = cz + size;

    v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    color = RED;
    let mut finger_vecs = vec![
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

    let mut old_idx = out_vec.len();
    out_vec.append(&mut finger_vecs); 
    idx_tup.push(("finger_4_top".to_string(), old_idx, out_vec.len()));

    // finger_4_bot
    size = 8.0;

    cx = 40.0; 
    cy = 0.0; 
    cz = 10.0;

    c1_x = cx - size * 1.2;
    c1_y = cy - size;
    c1_z = cz - size;

    c2_x = cx + size * 1.2;
    c2_y = cy + size;
    c2_z = cz + size;

    v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    color = YELLOW;
    let mut finger_vecs = vec![
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

    let mut old_idx = out_vec.len();
    out_vec.append(&mut finger_vecs); 
    idx_tup.push(("finger_4_bot".to_string(), old_idx, out_vec.len()));
    


    // palm
    size = 30.0;

    cx = 0.0; 
    cy = 9.0; 
    cz = 50.0;

    c1_x = cx - size;
    c1_y = cy - size;
    c1_z = cz - size / 1.5;

    c2_x = cx + size;
    c2_y = cy + size;
    c2_z = cz + size / 1.5;

    v_000 = V3 {x: c1_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_001 = V3 {x: c1_x as f64, y: c1_y as f64, z: c2_z as f64};
    v_010 = V3 {x: c1_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_100 = V3 {x: c2_x as f64, y: c1_y as f64, z: c1_z as f64};

    v_011 = V3 {x: c1_x as f64, y: c2_y as f64, z: c2_z as f64};
    v_110 = V3 {x: c2_x as f64, y: c2_y as f64, z: c1_z as f64};
    v_101 = V3 {x: c2_x as f64, y: c1_y as f64, z: c2_z as f64};

    v_111 = V3 {x: c2_x as f64, y: c2_y as f64, z: c2_z as f64};

    color = RED;
    let mut palm_vecs = vec![
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

    let mut old_idx = out_vec.len();
    out_vec.append(&mut palm_vecs);
    idx_tup.push(("palm".to_string(), old_idx, out_vec.len()));

    println!("{:?}", idx_tup); 
    return (out_vec, idx_tup); 
}

fn make_hand_skel(buffer: &mut [u32]) {
    make_line(buffer, 400, 500, 400, 360, GREEN); 
    make_line(buffer, 400, 360, 345, 300, GREEN); 
    make_line(buffer, 400, 360, 450, 300, GREEN); 
    make_line(buffer, 400, 360, 400, 300, GREEN); 

    make_line(buffer, 400, 400, 500, 400, GREEN); 
    make_line(buffer, 500, 400, 500, 350, GREEN); 

    make_line(buffer, 450, 200, 450, 300, GREEN); 
    make_line(buffer, 345, 200, 345, 300, GREEN); 
    make_line(buffer, 400, 200, 400, 300, GREEN); 
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

fn rotate_z(v: V3, angle: f64) -> V3 {
    let cos = angle.cos();
    let sin = angle.sin();
    V3 {
        x: v.x * cos - v.y * sin,
        y: v.x * sin + v.y * cos,
        z: v.z,
    }
}


fn main() {
    let mut buffer: Vec<u32> = reset_screen(); 

    let mut window = Window::new("Baby Steps", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| panic!("Error making a window! You goofed!: {}", e));
    
    let mut angle = 0.0;
    let mut hand = wireframe_hand(BLUE); 

    let mut hand_triangles = hand.0.clone(); 
    let mut dir = 1.0; 

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
        // let cube3 = rotate_triangles(get_cube_triangles(50, 50, 50, 50, BLUE), angle, -1.0 * angle, -0.5 * angle);
        // make_hand_skel(&mut buffer); 
        
        // tops
        let finger_1_top: Vec<_> = hand_triangles[24..36].to_vec();
        let finger_1_top = rotate_triangles(finger_1_top, 0.0, 0.0, 1.5 * angle);
        for (i, tri) in finger_1_top.into_iter().enumerate() {
            hand_triangles[24 + i] = tri;
        }

        let finger_2_top: Vec<_> = hand_triangles[48..60].to_vec();
        let finger_2_top = rotate_triangles(finger_2_top, 0.0, 0.0, 1.5 * angle);
        for (i, tri) in finger_2_top.into_iter().enumerate() {
            hand_triangles[48 + i] = tri;
        }

        let finger_3_top: Vec<_> = hand_triangles[72..84].to_vec();
        let finger_3_top = rotate_triangles(finger_3_top, 0.0, 0.0, 1.5 * angle);
        for (i, tri) in finger_3_top.into_iter().enumerate() {
            hand_triangles[72 + i] = tri;
        }

        let finger_4_top: Vec<_> = hand_triangles[96..108].to_vec();
        let finger_4_top = rotate_triangles(finger_4_top, 0.0, 0.0,  angle);
        for (i, tri) in finger_4_top.into_iter().enumerate() {
            hand_triangles[96 + i] = tri;
        }

        // bottoms
        let finger_1_bot: Vec<_> = hand_triangles[36..48].to_vec();
        let finger_1_bot = rotate_triangles(finger_1_bot, 0.0, 0.0, angle);
        for (i, tri) in finger_1_bot.into_iter().enumerate() {
            hand_triangles[36 + i] = tri;
        }

        let finger_2_bot: Vec<_> = hand_triangles[60..72].to_vec();
        let finger_2_bot = rotate_triangles(finger_2_bot, 0.0, 0.0, angle);
        for (i, tri) in finger_2_bot.into_iter().enumerate() {
            hand_triangles[60 + i] = tri;
        }

        let finger_3_bot: Vec<_> = hand_triangles[84..96].to_vec();
        let finger_3_bot = rotate_triangles(finger_3_bot, 0.0, 0.0, angle);
        for (i, tri) in finger_3_bot.into_iter().enumerate() {
            hand_triangles[84 + i] = tri;
        }

        let finger_4_bot: Vec<_> = hand_triangles[108..120].to_vec();
        let finger_4_bot = rotate_triangles(finger_4_bot, 0.0, 0.0, angle);
        for (i, tri) in finger_4_bot.into_iter().enumerate() {
            hand_triangles[108 + i] = tri;
        }

        // palm
        let palm: Vec<_> = hand_triangles[120..132].to_vec();
        let palm = rotate_triangles(palm, 0.0, 0.0, angle / 2.5);
        for (i, tri) in palm.into_iter().enumerate() {
            hand_triangles[120 + i] = tri;
        }

        // hand_triangles = rotate_triangles(hand_triangles.clone(),  angle, -1.0 * angle, -0.5 * angle);
        draw_3d_from_triangles(&mut buffer, hand_triangles.clone());
        
        angle += (dir * 0.00001);
        
        if angle > 0.001 {dir = -1.0; }
        if angle <= -0.00101 {dir = 1.0; }
        
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}