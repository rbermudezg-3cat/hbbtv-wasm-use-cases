use wasm_bindgen::prelude::*;
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

    // Add this method to Rect impl
    pub fn set_starting_point(&self, x: f64, y: f64) {
        let mut starting_point = STARTING_POINT.lock().unwrap();
        *starting_point = Some(Point { x, y });
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
pub fn get_entry_and_exit_points_from_point(
    dir: Direction,
    point: Point,
    candidate_rect: &Rect
) -> EntryExitPoints {
    // Convert the point to a Rect where all sides are at the point position
    let search_origin = Rect {
        top: point.y,
        bottom: point.y,
        left: point.x,
        right: point.x,
        width: 0.0,
        height: 0.0,
    };
    
    // Reuse the existing implementation for Rect
    get_entry_and_exit_points(dir, search_origin, candidate_rect)
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

#[wasm_bindgen]
pub fn get_absolute_distance(rect1: &Rect, rect2: &Rect, dir: Direction) -> f64 {
    // Obtener los puntos de entrada y salida
    let points = get_entry_and_exit_points(dir, *rect1, rect2);
    
    // Devolver la distancia absoluta en la dirección dir entre los puntos
    match dir {
        Direction::Left | Direction::Right => (points.entry_point.x - points.exit_point.x).abs(),
        Direction::Up | Direction::Down => (points.entry_point.y - points.exit_point.y).abs(),
    }
}

#[wasm_bindgen]
pub fn get_euclidean_distance(rect1: &Rect, rect2: &Rect, dir: Direction) -> f64 {
    // Obtener los puntos de entrada y salida
    let points = get_entry_and_exit_points(dir, *rect1, rect2);
    
    // Calcular la distancia entre los puntos
    let p1 = (points.entry_point.x - points.exit_point.x).abs();
    let p2 = (points.entry_point.y - points.exit_point.y).abs();
    
    // Devolver la distancia euclidiana entre P1 y P2
    (p1 * p1 + p2 * p2).sqrt()
}

#[wasm_bindgen]
pub fn get_distance_from_point(point: &Point, element: &Rect, dir: Direction) -> f64 {
    // Get exit point, entry point
    let points = get_entry_and_exit_points_from_point(dir, *point, element);
    
    // Find the points P1 inside the border box of starting point and P2 inside the border box of candidate
    // that minimize the distance between these two points
    let p1 = (points.entry_point.x - points.exit_point.x).abs();
    let p2 = (points.entry_point.y - points.exit_point.y).abs();
    
    // Return the euclidean distance between P1 and P2
    (p1 * p1 + p2 * p2).sqrt()
}

#[wasm_bindgen]
pub fn get_distance(rect1: &Rect, rect2: &Rect, dir: Direction) -> f64 {
    const K_ORTHOGONAL_WEIGHT_FOR_LEFT_RIGHT: f64 = 30.0;
    const K_ORTHOGONAL_WEIGHT_FOR_UP_DOWN: f64 = 2.0;
    const ALIGN_WEIGHT: f64 = 5.0;

    let mut orthogonal_bias = 0.0;
    let mut align_bias = 0.0;
    
    // Get exit point, entry point
    let points = get_entry_and_exit_points(dir, *rect1, rect2);
    
    // Find the points P1 inside the border box of starting point and P2 inside the border box of candidate
    // that minimize the distance between these two points
    let p1 = (points.entry_point.x - points.exit_point.x).abs();
    let p2 = (points.entry_point.y - points.exit_point.y).abs();
    
    // A: The euclidean distance between P1 and P2
    let a = (p1.powi(2) + p2.powi(2)).sqrt();
    
    // D: The intersection area between the border boxes
    let intersection_rect = get_intersection_rect(rect1, rect2);
    let d = intersection_rect.area;
    
    // Calculate B and C based on direction
    let (b, c) = match dir {
        Direction::Left | Direction::Right => {
            // If two elements are aligned, add align bias
            // else, add orthogonal bias
            if is_aligned(rect1, rect2, dir) {
                align_bias = (intersection_rect.height / rect1.height).min(1.0);
            } else {
                orthogonal_bias = rect1.height / 2.0;
            }
            
            let b_value = (p2 + orthogonal_bias) * K_ORTHOGONAL_WEIGHT_FOR_LEFT_RIGHT;
            let c_value = ALIGN_WEIGHT * align_bias;
            (b_value, c_value)
        },
        Direction::Up | Direction::Down => {
            // If two elements are aligned, add align bias
            // else, add orthogonal bias
            if is_aligned(rect1, rect2, dir) {
                align_bias = (intersection_rect.width / rect1.width).min(1.0);
            } else {
                orthogonal_bias = rect1.width / 2.0;
            }
            
            let b_value = (p1 + orthogonal_bias) * K_ORTHOGONAL_WEIGHT_FOR_UP_DOWN;
            let c_value = ALIGN_WEIGHT * align_bias;
            (b_value, c_value)
        }
    };
    
    // Return the final distance calculation
    a + b - c - d
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct IntersectionRect {
    pub width: f64,
    pub height: f64,
    pub area: f64,
}

#[wasm_bindgen]
impl IntersectionRect {
    #[wasm_bindgen(constructor)]
    pub fn new(width: f64, height: f64, area: f64) -> IntersectionRect {
        IntersectionRect { width, height, area }
    }
}

#[wasm_bindgen]
pub fn get_intersection_rect(rect1: &Rect, rect2: &Rect) -> IntersectionRect {
    let new_left = f64::max(rect1.left, rect2.left);
    let new_top = f64::max(rect1.top, rect2.top);
    let new_right = f64::min(rect1.right, rect2.right);
    let new_bottom = f64::min(rect1.bottom, rect2.bottom);
    
    let width = f64::max(0.0, new_right - new_left);
    let height = f64::max(0.0, new_bottom - new_top);
    
    let area = if width > 0.0 && height > 0.0 {
        (width * height).sqrt()
    } else {
        0.0
    };
    
    IntersectionRect { width, height, area }
}

// Add these new functions after the existing ones

#[wasm_bindgen]
pub fn get_closest_element(
    current_element_rect: &Rect,
    candidate_rects: Vec<Rect>,
    dir: Direction,
    distance_function_name: &str
) -> Option<usize> {
    if candidate_rects.is_empty() {
        return None;
    }

    let event_target_rect = current_element_rect.clone();
    let mut min_distance = f64::INFINITY;
    let mut min_distance_indices = Vec::new();

    // Choose the appropriate distance function
    let distance_function = match distance_function_name {
        "getAbsoluteDistance" => get_absolute_distance,
        "getEuclideanDistance" => get_euclidean_distance,
        "getDistance" => get_distance,
        "getDistanceFromPoint" => |r1: &Rect, r2: &Rect, d: Direction| {
            // Get the starting point from global variable
            let starting_point = {
                let sp = STARTING_POINT.lock().unwrap();
                match *sp {
                    Some(point) => point,
                    None => Point { x: 0.0, y: 0.0 }
                }
            };
            get_distance_from_point(&starting_point, r2, d)
        },
        "getInnerDistance" => |r1: &Rect, r2: &Rect, d: Direction| {
            match d {
                Direction::Left => r1.right - r2.right,
                Direction::Right => r2.left - r1.left,
                Direction::Up => r1.bottom - r2.bottom,
                Direction::Down => r2.top - r1.top
            }.abs()
        },
        _ => get_distance // Default to getDistance
    };

    // Find the closest candidate(s)
    for (i, candidate_rect) in candidate_rects.iter().enumerate() {
        let distance = distance_function(&event_target_rect, candidate_rect, dir);

        if distance < min_distance {
            min_distance = distance;
            min_distance_indices.clear();
            min_distance_indices.push(i);
        } else if distance == min_distance {
            min_distance_indices.push(i);
        }
    }

    // If multiple candidates have the same minimum distance
    if min_distance_indices.len() > 1 && distance_function_name == "getAbsoluteDistance" {
        // Recursive call with getEuclideanDistance to break ties
        let tied_candidates: Vec<Rect> = min_distance_indices
            .iter()
            .map(|&i| candidate_rects[i].clone())
            .collect();
        
        if let Some(closest_index) = get_closest_element(
            current_element_rect,
            tied_candidates,
            dir,
            "getEuclideanDistance"
        ) {
            return Some(min_distance_indices[closest_index]);
        }
    }

    // Return the first (or only) closest candidate
    min_distance_indices.first().copied()
}

#[wasm_bindgen]
pub fn select_best_candidate_from_edge(current_elm_rect: &Rect, candidate_rects: Vec<Rect>, dir: Direction) -> Option<usize> {
    let starting_point_exists = {
        let sp = STARTING_POINT.lock().unwrap();
        sp.is_some()
    };
    
    if starting_point_exists {
        // If starting point exists, use getDistanceFromPoint
        get_closest_element(current_elm_rect, candidate_rects, dir, "getDistanceFromPoint")
    } else {
        // Otherwise use getInnerDistance
        get_closest_element(current_elm_rect, candidate_rects, dir, "getInnerDistance")
    }
}

#[wasm_bindgen]
pub fn select_best_candidate(
    current_elm: &Rect,
    candidates: Vec<Rect>,
    dir: Direction,
    spatial_navigation_function: &str
) -> Option<usize> {
    if candidates.is_empty() {
        return None;
    }

    let mut aligned_candidates = Vec::new();
    
    // Apply spatial navigation function logic
    match spatial_navigation_function {
        "grid" => {
            // Filter candidates that are aligned with the current element
            for (i, candidate) in candidates.iter().enumerate() {
                if is_aligned(current_elm, candidate, dir) {
                    aligned_candidates.push(i);
                }
            }
            
            // If we have aligned candidates, use them instead of all candidates
            if !aligned_candidates.is_empty() {
                let filtered_candidates: Vec<Rect> = aligned_candidates
                    .iter()
                    .map(|&i| candidates[i].clone())
                    .collect();
                
                // Get closest element using absolute distance
                if let Some(closest_index) = get_closest_element(
                    current_elm,
                    filtered_candidates,
                    dir,
                    "getAbsoluteDistance"
                ) {
                    return Some(aligned_candidates[closest_index]);
                }
            }
            
            // Fall back to absolute distance if no aligned candidates
            get_closest_element(current_elm, candidates, dir, "getAbsoluteDistance")
        },
        _ => {
            // Default case - use standard distance function
            get_closest_element(current_elm, candidates, dir, "getDistance")
        }
    }
}

// Initialize function for panic hook
#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// Add this to lib.rs
#[wasm_bindgen]
pub fn is_a_tag_without_href(tag_name: &str, href_attr: Option<String>, tab_index_attr: Option<String>) -> bool {
    tag_name == "A" && href_attr.is_none() && tab_index_attr.is_none()
}

// Add this after other public functions

#[wasm_bindgen]
pub fn read_css_var(element_style: &str, var_name: &str) -> String {
    let property_name = format!("--{}", var_name);
    let property_pattern = format!("{}\\s*:\\s*([^;]+)", regex::escape(&property_name));
    
    if let Ok(re) = regex::Regex::new(&property_pattern) {
        if let Some(captures) = re.captures(element_style) {
            if let Some(value) = captures.get(1) {
                return value.as_str().trim().to_string();
            }
        }
    }
    
    String::new() // Return empty string if property not found
}