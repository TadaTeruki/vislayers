use geometry::FocusRange;
use geometry::Rect;
use gtk4::cairo::Context;
use gtk4::glib::timeout_add_local;
use gtk4::glib::timeout_add_seconds_local;
use gtk4::glib::ControlFlow;
use gtk4::glib::ExitCode;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::DrawingArea;
use gtk4::GestureClick;
use gtk4::GestureDrag;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use worley_particle::map::IDWStrategy;
use worley_particle::map::InterpolationMethod;
use worley_particle::map::IsobandResult;
use worley_particle::map::ParticleMap;

// struct Instance {
//     isobands: Vec<IsobandResult>,
// }

// impl Instance {
//     fn new() -> Self {
//         let terrain_map = ParticleMap::<f64>::read_from_file("./data/1005765147554121537.particlemap")
//             .expect("Error reading terrain map");

//         let thresholds = (0..=30).map(|i| i as f64 * 0.75 / 30.).collect::<Vec<_>>();

//         let isobands = terrain_map
//             .isobands(
//                 terrain_map.corners(),
//                 500000.0,
//                 &thresholds,
//                 &InterpolationMethod::IDW(IDWStrategy::default_from_params(terrain_map.params())),
//                 true,
//             )
//             .expect("Error generating isobands");

//         Self { isobands }
//     }

//     fn draw_fn(&self, drawing_area: &DrawingArea, cr: &Context) {
//         let width = drawing_area.width();
//         let height = drawing_area.height();

//         cr.set_source_rgb(1.0, 1.0, 1.0);
//         cr.paint();

//         cr.set_source_rgb(0.0, 0.0, 0.0);
//         cr.set_line_width(1.0);

//         for isoband in &self.isobands {
//             // cr.move_to(isoband.p1.x, isoband.p1.y);
//             // cr.line_to(isoband.p2.x, isoband.p2.y);
//             // cr.stroke();
//         }
//     }
// }

mod colormap;
mod geometry;

// zoom(dZoomLevel: number) {
//     this.zoomLevel += dZoomLevel;
//     this.focusRange.setRadius(Math.pow(2, this.zoomLevel));
// }

// moveFocus(dx: number, dy: number) {
//     this.focusRange.moveCenter(
//         -dx * this.moveSpeed * this.focusRange.radius,
//         -dy * this.moveSpeed * this.focusRange.radius,
//     );
//     this.update();
// }

struct MapState {
    focus_range: FocusRange,
    zoom_level: f64,
    move_speed: f64,
    grid_thresholds: Vec<f64>,
    grid_interval: f64,
}

impl MapState {
    fn new(grid_start: i32, grid_end: i32, grid_interval: f64) -> Self {
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

    fn zoom(&mut self, d_zoom_level: f64) {
        self.zoom_level += d_zoom_level;
        self.focus_range.set_radius(2.0_f64.powf(self.zoom_level));
    }

    fn move_focus(&mut self, dx: f64, dy: f64) {
        self.focus_range.move_center(
            -dx * self.move_speed * self.focus_range.radius(),
            -dy * self.move_speed * self.focus_range.radius(),
        );
    }

    fn draw_grid(&self, drawing_area: &DrawingArea, cr: &Context) {
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
                let ix = rect.map_coord_x(x as f64, 0.0, area_width);
                cr.move_to(ix, rect.map_coord_y(rect.min_y, 0.0, area_height));
                cr.line_to(ix, rect.map_coord_y(rect.max_y, 0.0, area_height));
                cr.stroke().expect("Error drawing grid");
                x += grid_threshold;
            }

            let grid_start_y = (rect.min_y / grid_threshold).floor() * grid_threshold;
            let grid_end_y = (rect.max_y / grid_threshold).ceil() * grid_threshold;

            let mut y = grid_start_y;
            while y <= grid_end_y {
                let iy = rect.map_coord_y(y as f64, 0.0, area_height);
                cr.move_to(rect.map_coord_x(rect.min_x, 0.0, area_width), iy);
                cr.line_to(rect.map_coord_x(rect.max_x, 0.0, area_width), iy);
                cr.stroke().expect("Error drawing grid");
                y += grid_threshold;
            }
        }
    }

    fn update(&mut self, drawing_area: &DrawingArea, cr: &Context) {
        self.focus_range.update();
        self.draw_grid(drawing_area, cr);
    }
}

fn main() -> ExitCode {
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(400)
            .title("Visualizer")
            .build();
        let drawing_area = DrawingArea::new();

        let state = Rc::new(RefCell::new(MapState::new(-2, 2, 10.0)));

        drawing_area.set_draw_func({
            let state = Rc::clone(&state);
            move |drawing_area, cr, _, _| {
                let mut state = state.borrow_mut();
                state.update(drawing_area, cr);
            }
        });

        window.set_child(Some(&drawing_area));

        let gesture_drag = GestureDrag::new();
        let last_position = Rc::new(RefCell::new(None));
        gesture_drag.connect_drag_update({
            let state = Rc::clone(&state);
            let last_position = Rc::clone(&last_position);
            move |_, x, y| {
                let mut last_position = last_position.borrow_mut();
                let (dx, dy) = match *last_position {
                    Some((last_x, last_y)) => (x - last_x, y - last_y),
                    None => (0.0, 0.0),
                };
                *last_position = Some((x, y));
                let mut state = state.borrow_mut();
                state.move_focus(dx, dy);
            }
        });

        gesture_drag.connect_drag_end({
            let last_position = Rc::clone(&last_position);
            move |_, _, _| {
                *last_position.borrow_mut() = None;
            }
        });

        drawing_area.add_controller(gesture_drag);

        window.present();

        let tick = move || {
            drawing_area.queue_draw();
            ControlFlow::Continue
        };
        timeout_add_local(Duration::from_millis(1000 / 60), tick);
    });

    app.run()
}
