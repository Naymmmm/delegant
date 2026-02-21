use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};
use rand::Rng;

use crate::error::{AppError, AppResult};

fn new_enigo() -> AppResult<Enigo> {
    Enigo::new(&Settings::default()).map_err(|e| AppError::Input(format!("Enigo init: {}", e)))
}

/// Fast WindMouse algorithm — generates human-like curved mouse paths, but highly optimized for speed
/// Returns Vec<[i32; 3]> where each element is [x, y, wait_ms].
fn windmouse_points(start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Vec<[i32; 3]> {
    let mut rng = rand::rng();

    let gravity = 9.0;
    let wind = 3.0;
    // VERY low wait times for speed
    let min_wait = 1.0;
    let max_wait = 2.0;
    // HIGH max step to cover screen instantly while retaining slight curvature
    let max_step: f64 = 60.0;
    let target_area = 15.0;
    // HIGH mouse speed
    let mouse_speed = (rng.random::<f64>() * 10.0 + 30.0).max(1.0);

    let mut points: Vec<[i32; 3]> = Vec::new();
    let mut cx = start_x;
    let mut cy = start_y;
    let mut vx = 0.0_f64;
    let mut vy = 0.0_f64;
    let mut wx = 0.0_f64;
    let mut wy = 0.0_f64;

    let sqrt2 = 2.0_f64.sqrt();
    let sqrt3 = 3.0_f64.sqrt();
    let sqrt5 = 5.0_f64.sqrt();

    loop {
        let dx = end_x - cx;
        let dy = end_y - cy;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 1.0 {
            break;
        }

        // Wind: more random far from target, converges near target
        if dist >= target_area {
            wx = wx / sqrt3 + (rng.random::<f64>() * wind * 2.0 - wind) / sqrt5;
            wy = wy / sqrt3 + (rng.random::<f64>() * wind * 2.0 - wind) / sqrt5;
        } else {
            wx /= sqrt2;
            wy /= sqrt2;
            if max_step >= 3.0 {
                let factor = rng.random::<f64>() * 3.0 + 3.0;
                wx += (factor * (rng.random::<f64>() * 2.0 - 1.0)) / dist.max(0.1);
                wy += (factor * (rng.random::<f64>() * 2.0 - 1.0)) / dist.max(0.1);
            }
        }

        // Gravity pulls toward target
        vx += wx + gravity * dx / dist;
        vy += wy + gravity * dy / dist;

        // Clamp velocity
        let vel_mag = (vx * vx + vy * vy).sqrt();
        let step = max_step.min(dist);
        if vel_mag > step {
            let scale = step / vel_mag * (rng.random::<f64>() * 0.2 + 0.9);
            vx *= scale;
            vy *= scale;
        }

        cx += vx;
        cy += vy;

        let wait = ((dist / mouse_speed).round()).clamp(min_wait, max_wait) as i32;
        points.push([cx.round() as i32, cy.round() as i32, wait]);
    }

    points
}

fn fast_smooth_move_to(enigo: &mut Enigo, x: i32, y: i32) -> AppResult<()> {
    let (cur_x, cur_y) = enigo
        .location()
        .map_err(|e| AppError::Input(format!("cursor location: {}", e)))?;

    // Skip smooth move for tiny distances, instantly jump and return
    let dx = (x - cur_x) as f64;
    let dy = (y - cur_y) as f64;
    if (dx * dx + dy * dy).sqrt() < 3.0 {
        enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| AppError::Input(e.to_string()))?;
        return Ok(());
    }

    let points = windmouse_points(cur_x as f64, cur_y as f64, x as f64, y as f64);

    for point in &points {
        enigo
            .move_mouse(point[0], point[1], Coordinate::Abs)
            .map_err(|e| AppError::Input(e.to_string()))?;
        let wait = point[2].max(0) as u64;
        if wait > 0 {
            std::thread::sleep(std::time::Duration::from_millis(wait));
        }
    }

    // Ensure exact landing
    enigo
        .move_mouse(x, y, Coordinate::Abs)
        .map_err(|e| AppError::Input(e.to_string()))?;
    Ok(())
}

pub fn move_to(x: i32, y: i32) -> AppResult<()> {
    let mut enigo = new_enigo()?;
    fast_smooth_move_to(&mut enigo, x, y)
}

pub fn click(x: i32, y: i32, button: &str) -> AppResult<()> {
    let mut enigo = new_enigo()?;
    fast_smooth_move_to(&mut enigo, x, y)?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    let btn = match button {
        "right" => Button::Right,
        "middle" => Button::Middle,
        _ => Button::Left,
    };
    enigo
        .button(btn, Direction::Click)
        .map_err(|e| AppError::Input(e.to_string()))
}

pub fn double_click(x: i32, y: i32) -> AppResult<()> {
    let mut enigo = new_enigo()?;
    fast_smooth_move_to(&mut enigo, x, y)?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    enigo
        .button(Button::Left, Direction::Click)
        .map_err(|e| AppError::Input(e.to_string()))?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    enigo
        .button(Button::Left, Direction::Click)
        .map_err(|e| AppError::Input(e.to_string()))
}

pub fn scroll(x: i32, y: i32, clicks: i32) -> AppResult<()> {
    let mut enigo = new_enigo()?;
    fast_smooth_move_to(&mut enigo, x, y)?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    enigo
        .scroll(clicks, enigo::Axis::Vertical)
        .map_err(|e| AppError::Input(e.to_string()))
}

pub fn drag(start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> AppResult<()> {
    let mut enigo = new_enigo()?;
    fast_smooth_move_to(&mut enigo, start_x, start_y)?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    enigo
        .button(Button::Left, Direction::Press)
        .map_err(|e| AppError::Input(e.to_string()))?;
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Use fast windmouse for the drag path
    let points = windmouse_points(start_x as f64, start_y as f64, end_x as f64, end_y as f64);

    for point in &points {
        enigo
            .move_mouse(point[0], point[1], Coordinate::Abs)
            .map_err(|e| AppError::Input(e.to_string()))?;
        let wait = point[2].max(0) as u64;
        if wait > 0 {
            std::thread::sleep(std::time::Duration::from_millis(wait));
        }
    }

    // Ensure exact landing
    enigo
        .move_mouse(end_x, end_y, Coordinate::Abs)
        .map_err(|e| AppError::Input(e.to_string()))?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    enigo
        .button(Button::Left, Direction::Release)
        .map_err(|e| AppError::Input(e.to_string()))
}
