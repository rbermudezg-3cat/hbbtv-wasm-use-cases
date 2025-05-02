use wasm_bindgen::prelude::*;
use js_sys::Math;
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen()]
    fn startDebugTimer(s: &str);
    fn endDebugTimer(s: &str);
}

// Direction enum
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Right, 
    Up,
    Down,
}

// DOM Rectangle representation in Rust
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Rect {
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct EntryExitPoints {
    pub entry_point: Point,
    pub exit_point: Point,
}

// Add this near the top of your file
static HTML_BUFFER: Lazy<Mutex<String>> = Lazy::new(|| {
    Mutex::new(String::with_capacity(10000)) // Initial capacity, adjust as needed
});

// Track the starting position for the color shift
static COLOR_OFFSET: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));

// Variable global para almacenar el punto inicial
static STARTING_POINT: Lazy<Mutex<Option<Point>>> = Lazy::new(|| Mutex::new(None));

#[wasm_bindgen]
pub fn render_navigable_elements(count: u32) -> String {
    startDebugTimer("renderElementsWithWasm");
    let mut html = HTML_BUFFER.lock().unwrap();
    html.clear(); // Clear previous content
    
    // Get and update the color offset
    let mut offset = COLOR_OFFSET.lock().unwrap();
    *offset = (*offset + 1) % count; // Increment and wrap around
    
    // Generate the specified number of items
    for i in 1..=count {
        // Calculate color with shifting effect
        let position = (i + *offset) % count;
        let factor = position as f32 / count as f32;
        let color_value = (factor * 255.0) as u8;
        let color = format!("rgb({0}, {0}, {0})", color_value);
        
        html.push_str(&format!(
            "<div class=\"item\" tabindex=\"0\" style=\"background-color: {}; color: {}\">{}</div>", 
            color, 
            if factor < 0.5 { "white" } else { "black" }, // Text color for contrast
            i
        ));
    }
    
    endDebugTimer("renderElementsWithWasm");
    html.clone() // Return a clone of the buffer
}

#[wasm_bindgen]
impl Rect {
    #[wasm_bindgen(constructor)]
    pub fn new(width: f64, height: f64, top: f64, right: f64, bottom: f64, left: f64) -> Rect {
        Rect { width, height, top, right, bottom, left }
    }

    pub fn from_dom_rect(rect: &web_sys::DomRect) -> Rect {
        Rect {
            left: rect.left(),
            top: rect.top(),
            right: rect.right(),
            bottom: rect.bottom(),
            width: rect.width(),
            height: rect.height(),
        }
    }
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
}

#[wasm_bindgen]
pub fn is_below(rect1: &Rect, rect2: &Rect) -> bool {
    rect1.top >= rect2.bottom || (rect1.top >= rect2.top && rect1.bottom > rect2.bottom && rect1.left < rect2.right && rect1.right > rect2.left)
}

#[wasm_bindgen]
pub fn is_right_side(rect1: &Rect, rect2: &Rect) -> bool {
    rect1.left >= rect2.right || 
    (rect1.left >= rect2.left && 
     rect1.right > rect2.right && 
     rect1.bottom > rect2.top && 
     rect1.top < rect2.bottom)
}

#[wasm_bindgen]
pub fn is_outside(rect1: &Rect, rect2: &Rect, dir: Direction) -> bool {
    match dir {
        Direction::Left => is_right_side(rect2, rect1),
        Direction::Right => is_right_side(rect1, rect2),
        Direction::Up => is_below(rect2, rect1),
        Direction::Down => is_below(rect1, rect2),
    }
}

#[wasm_bindgen]
pub fn is_inside(container: &Rect, child: &Rect) -> bool {
    let right_edge_check = container.left < child.right && container.right >= child.right;
    let left_edge_check = container.left <= child.left && container.right > child.left;
    let top_edge_check = container.top <= child.top && container.bottom > child.top;
    let bottom_edge_check = container.top < child.bottom && container.bottom >= child.bottom;

    (right_edge_check || left_edge_check) && (top_edge_check || bottom_edge_check)
}

#[wasm_bindgen]
pub fn is_aligned(rect1: &Rect, rect2: &Rect, dir: Direction) -> bool {
    match dir {
        Direction::Left | Direction::Right => rect1.bottom > rect2.top && rect1.top < rect2.bottom,
        Direction::Up | Direction::Down => rect1.right > rect2.left && rect1.left < rect2.right,
    }
}

#[wasm_bindgen]
pub fn get_entry_and_exit_points(
    dir: Direction,
    search_origin: Rect,
    candidate_rect: &Rect,
) -> EntryExitPoints {
    let mut entry_point = Point { x: 0.0, y: 0.0 };
    let mut exit_point = Point { x: 0.0, y: 0.0 };

    // Obtener el punto inicial global si existe
    let starting_point = {
        let sp = STARTING_POINT.lock().unwrap();
        *sp
    };

    if let Some(origin) = starting_point {
        // Caso con startingPoint global
        exit_point = origin;

        match dir {
            Direction::Left => entry_point.x = candidate_rect.right,
            Direction::Up => entry_point.y = candidate_rect.bottom,
            Direction::Right => entry_point.x = candidate_rect.left,
            Direction::Down => entry_point.y = candidate_rect.top,
        }

        match dir {
            Direction::Left | Direction::Right => {
                if origin.y <= candidate_rect.top {
                    entry_point.y = candidate_rect.top;
                } else if origin.y < candidate_rect.bottom {
                    entry_point.y = origin.y;
                } else {
                    entry_point.y = candidate_rect.bottom;
                }
            }
            Direction::Up | Direction::Down => {
                if origin.x <= candidate_rect.left {
                    entry_point.x = candidate_rect.left;
                } else if origin.x < candidate_rect.right {
                    entry_point.x = origin.x;
                } else {
                    entry_point.x = candidate_rect.right;
                }
            }
        }
    } else {
        // Set direction - Caso sin startingPoint usando searchOrigin
        match dir {
            Direction::Left => {
                exit_point.x = search_origin.left;
                entry_point.x = if candidate_rect.right < search_origin.left {
                    candidate_rect.right
                } else {
                    search_origin.left
                };
            }
            Direction::Up => {
                exit_point.y = search_origin.top;
                entry_point.y = if candidate_rect.bottom < search_origin.top {
                    candidate_rect.bottom
                } else {
                    search_origin.top
                };
            }
            Direction::Right => {
                exit_point.x = search_origin.right;
                entry_point.x = if candidate_rect.left > search_origin.right {
                    candidate_rect.left
                } else {
                    search_origin.right
                };
            }
            Direction::Down => {
                exit_point.y = search_origin.bottom;
                entry_point.y = if candidate_rect.top > search_origin.bottom {
                    candidate_rect.top
                } else {
                    search_origin.bottom
                };
            }
        }

        // Set orthogonal direction
        match dir {
            Direction::Left | Direction::Right => {
                if is_below(&search_origin, candidate_rect) {
                    exit_point.y = search_origin.top;
                    entry_point.y = if candidate_rect.bottom < search_origin.top {
                        candidate_rect.bottom
                    } else {
                        search_origin.top
                    };
                } else if is_below(candidate_rect, &search_origin) {
                    exit_point.y = search_origin.bottom;
                    entry_point.y = if candidate_rect.top > search_origin.bottom {
                        candidate_rect.top
                    } else {
                        search_origin.bottom
                    };
                } else {
                    // Ni uno es below del otro - encontrar el punto máximo
                    exit_point.y = f64::max(search_origin.top, candidate_rect.top);
                    entry_point.y = exit_point.y;
                }
            }
            Direction::Up | Direction::Down => {
                if is_right_side(&search_origin, candidate_rect) {
                    exit_point.x = search_origin.left;
                    entry_point.x = if candidate_rect.right < search_origin.left {
                        candidate_rect.right
                    } else {
                        search_origin.left
                    };
                } else if is_right_side(candidate_rect, &search_origin) {
                    exit_point.x = search_origin.right;
                    entry_point.x = if candidate_rect.left > search_origin.right {
                        candidate_rect.left
                    } else {
                        search_origin.right
                    };
                } else {
                    // Ni uno es right_side del otro - encontrar el punto máximo
                    exit_point.x = f64::max(search_origin.left, candidate_rect.left);
                    entry_point.x = exit_point.x;
                }
            }
        }
    }

    EntryExitPoints {
        entry_point,
        exit_point,
    }
}

#[wasm_bindgen]
pub fn set_starting_point(x: f64, y: f64) {
    let mut starting_point = STARTING_POINT.lock().unwrap();
    *starting_point = Some(Point { x, y });
}

#[wasm_bindgen]
pub fn get_starting_point() -> Option<Point> {
    let starting_point = STARTING_POINT.lock().unwrap();
    *starting_point
}

#[wasm_bindgen]
pub fn clear_starting_point() {
    let mut starting_point = STARTING_POINT.lock().unwrap();
    *starting_point = None;
}

// Initialize function for panic hook
#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}