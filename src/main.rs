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
use gtk4::Gesture;
use gtk4::GestureClick;
use gtk4::GestureDrag;
use gtk4::GestureSingle;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use view::ViewState;
use worley_particle::map::IDWStrategy;
use worley_particle::map::InterpolationMethod;
use worley_particle::map::IsobandResult;
use worley_particle::map::ParticleMap;

struct MapData {
    isobands: Vec<IsobandResult>,
}

impl MapData {
    fn new() -> Self {
        let terrain_map =
            ParticleMap::<f64>::read_from_file("./data/1005765147554121537.particlemap")
                .expect("Error reading terrain map");

        let thresholds = (0..=30).map(|i| i as f64 * 0.75 / 30.).collect::<Vec<_>>();

        let isobands = terrain_map
            .isobands(
                terrain_map.corners(),
                500000.0,
                &thresholds,
                &InterpolationMethod::IDW(IDWStrategy::default_from_params(terrain_map.params())),
                true,
            )
            .expect("Error generating isobands");

        Self { isobands }
    }

    fn draw_fn(&self, drawing_area: &DrawingArea, cr: &Context) {}
}

mod colormap;
mod geometry;
mod view;

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

        let view = Rc::new(RefCell::new(ViewState::new(-2, 2, 10.0)));

        drawing_area.set_draw_func({
            let view = Rc::clone(&view);
            move |drawing_area, cr, _, _| {
                let mut view = view.borrow_mut();
                view.update(drawing_area, cr);
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
