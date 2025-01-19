use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};

use crate::geometry::FocusRange;

pub struct ViewState {
    focus_range: FocusRange,
    zoom_level: f64,
    move_speed: f64,
    grid_thresholds: Vec<f64>,
    grid_interval: f64,
}

impl ViewState {
    pub fn new(grid_start: i32, grid_end: i32, grid_interval: f64) -> Self {
        let grid_thresholds = {
            let mut grid_thresholds = Vec::new();
            for i in grid_start..grid_end {
                grid_thresholds.push(grid_interval.powi(i));
            }
            grid_thresholds
        };
        Self {
            focus_range: FocusRange::new(0.0, 0.0, 1.0, 0.01),
            zoom_level: 0.0,
            move_speed: 0.1,
            grid_thresholds,
            grid_interval,
        }
    }

    pub fn focus_range(&self) -> &FocusRange {
        &self.focus_range
    }

    pub fn zoom(&mut self, d_zoom_level: f64) {
        self.zoom_level += d_zoom_level * 0.25;
        self.focus_range.set_radius(2.0_f64.powf(self.zoom_level));
    }

    pub fn move_focus(&mut self, dx: f64, dy: f64) {
        self.focus_range.move_center(
            -dx * self.move_speed * self.focus_range.radius(),
            -dy * self.move_speed * self.focus_range.radius(),
        );
    }

    pub fn draw_grid(&self, drawing_area: &DrawingArea, cr: &Context) {
        for grid_threshold in &self.grid_thresholds {
            let x_factor =
                (self.focus_range.radius() / grid_threshold / self.grid_interval).log2() * 0.4;
            let alpha = 1.0 - x_factor * x_factor;

            if alpha < 0.0 {
                continue;
            }

            let area_width = drawing_area.width() as f64;
            let area_height = drawing_area.height() as f64;

            let rect = self.focus_range.to_rect(area_width, area_height);

            cr.set_source_rgba(0.0, 0.333, 0.533, alpha);
            cr.set_line_width(1.0);

            let grid_start_x = (rect.min_x / grid_threshold).floor() * grid_threshold;
            let grid_end_x = (rect.max_x / grid_threshold).ceil() * grid_threshold;

            let mut x = grid_start_x;
            while x <= grid_end_x {
                let ix = rect.map_coord_x(x, 0.0, area_width);
                cr.move_to(ix, rect.map_coord_y(rect.min_y, 0.0, area_height));
                cr.line_to(ix, rect.map_coord_y(rect.max_y, 0.0, area_height));
                cr.stroke().expect("Error drawing grid");
                x += grid_threshold;
            }

            let grid_start_y = (rect.min_y / grid_threshold).floor() * grid_threshold;
            let grid_end_y = (rect.max_y / grid_threshold).ceil() * grid_threshold;

            let mut y = grid_start_y;
            while y <= grid_end_y {
                let iy = rect.map_coord_y(y, 0.0, area_height);
                cr.move_to(rect.map_coord_x(rect.min_x, 0.0, area_width), iy);
                cr.line_to(rect.map_coord_x(rect.max_x, 0.0, area_width), iy);
                cr.stroke().expect("Error drawing grid");
                y += grid_threshold;
            }
        }
    }

    pub fn update(&mut self) {
        self.focus_range.update();
    }
}
