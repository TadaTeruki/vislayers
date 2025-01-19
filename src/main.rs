use colormap::SimpleColorMap;
use geometry::FocusRange;
use gtk4::cairo::Context;
use gtk4::glib::timeout_add_local;
use gtk4::glib::ControlFlow;
use gtk4::glib::ExitCode;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::DrawingArea;
use gtk4::EventControllerScroll;
use gtk4::EventControllerScrollFlags;
use gtk4::GestureDrag;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use view::ViewState;
use worley_particle::map::IDWStrategy;
use worley_particle::map::InterpolationMethod;
use worley_particle::map::IsobandResult;
use worley_particle::map::ParticleMap;

struct TerrainMap {
    #[allow(dead_code)]
    particle_map: ParticleMap<f64>,
    isobands: Vec<IsobandResult>,
}

impl TerrainMap {
    fn new() -> Self {
        let particle_map =
            ParticleMap::<f64>::read_from_file("./data/1005765147554121537.particlemap")
                .expect("Error reading terrain map");

        let num_thresholds = 20;

        let thresholds = (0..num_thresholds)
            .map(|i| i as f64 * 0.9 / (num_thresholds - 1) as f64)
            .collect::<Vec<_>>();

        let isobands = particle_map
            .isobands(
                particle_map.corners(),
                200000.0,
                &thresholds,
                &InterpolationMethod::IDW(IDWStrategy::default_from_params(particle_map.params())),
                true,
            )
            .expect("Error generating isobands");

        Self {
            particle_map,
            isobands,
        }
    }

    fn draw_fn(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        let color_map = SimpleColorMap::new(
            vec![
                [100.0, 150.0, 70.0],
                [60.0, 90.0, 55.0],
                [210.0, 210.0, 210.0],
            ],
            vec![0.0, 0.35, 0.6],
        );

        let rect = focus_range.to_rect(drawing_area.width() as f64, drawing_area.height() as f64);

        let isobands_step = (focus_range.radius() * 2.0).ceil() as usize;

        for threshold in self.isobands.iter().step_by(isobands_step) {
            let color = color_map.get_color(threshold.threshold);
            cr.set_source_rgb(color[0] / 255.0, color[1] / 255.0, color[2] / 255.0);
            cr.new_path();
            for polygon in &threshold.polygons {
                for (i, point) in polygon.iter().enumerate() {
                    let x = rect.map_coord_x(point.0, 0.0, drawing_area.width() as f64);
                    let y = rect.map_coord_y(point.1, 0.0, drawing_area.height() as f64);
                    if i == 0 {
                        cr.move_to(x, y);
                    } else {
                        cr.line_to(x, y);
                    }
                }
                cr.close_path();
            }
            cr.fill().expect("Failed to fill polygon");
        }

        cr.set_source_rgb(1.0, 0.0, 0.0);
        cr.arc(
            rect.map_coord_x(0.0, 0.0, drawing_area.width() as f64),
            rect.map_coord_y(0.0, 0.0, drawing_area.height() as f64),
            2.0,
            0.0,
            2.0 * std::f64::consts::PI,
        );
        cr.fill().expect("Failed to draw center point");
    }
}

mod colormap;
mod geometry;
mod view;

fn main() -> ExitCode {
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();

    let terrain_map = Rc::new(RefCell::new(TerrainMap::new()));

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(400)
            .title("Visualizer")
            .build();

        let drawing_area = DrawingArea::new();

        let view = Rc::new(RefCell::new(ViewState::new(-2, 2, 10.0)));

        drawing_area.set_draw_func({
            let view = Rc::clone(&view);
            let terrain_map = Rc::clone(&terrain_map);
            move |drawing_area, cr, _, _| {
                cr.set_source_rgb(50.0 / 255.0, 110.0 / 255.0, 150.0 / 255.0);
                cr.paint().expect("Failed to paint background");

                let mut view = view.borrow_mut();
                view.update();
                view.draw_grid(drawing_area, cr);

                let terrain_map = terrain_map.borrow();
                let focus_range = view.focus_range();
                terrain_map.draw_fn(drawing_area, cr, focus_range);
            }
        });

        window.set_child(Some(&drawing_area));

        let gesture_drag = GestureDrag::new();
        let last_position = Rc::new(RefCell::new(None));
        gesture_drag.connect_drag_update({
            let view = Rc::clone(&view);
            let last_position = Rc::clone(&last_position);
            move |_, x, y| {
                let mut last_position = last_position.borrow_mut();
                let (dx, dy) = match *last_position {
                    Some((last_x, last_y)) => (x - last_x, y - last_y),
                    None => (0.0, 0.0),
                };
                *last_position = Some((x, y));
                let mut view = view.borrow_mut();
                view.move_focus(dx, dy);
            }
        });

        gesture_drag.connect_drag_end({
            let last_position = Rc::clone(&last_position);
            move |_, _, _| {
                *last_position.borrow_mut() = None;
            }
        });

        let event_controller_scroll = EventControllerScroll::new(EventControllerScrollFlags::all());
        event_controller_scroll.connect_scroll({
            let view = Rc::clone(&view);
            move |_, _, dy| {
                let mut view = view.borrow_mut();
                view.zoom(dy);
                Propagation::Stop
            }
        });

        drawing_area.add_controller(gesture_drag);
        drawing_area.add_controller(event_controller_scroll);

        window.present();

        let tick = move || {
            drawing_area.queue_draw();
            ControlFlow::Continue
        };
        timeout_add_local(Duration::from_millis(1000 / 60), tick);
    });

    app.run()
}
