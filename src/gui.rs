pub fn world_to_screen(
    world_x: f64,
    world_y: f64,
    cam_x: f64,
    cam_y: f64,
    zoom: f64,
    center_x: f64,
    center_y: f64,
) -> (i32, i32) {
    // screen = (world - cam) * zoom + center
    let screen_x = ((world_x - cam_x) * zoom + center_x) as i32;
    let screen_y = ((world_y - cam_y) * zoom + center_y) as i32;
    (screen_x, screen_y)
}
