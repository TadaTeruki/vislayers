use std::{cell::RefCell, rc::Rc, time::Duration};

use gtk4::{
    cairo::Context,
    glib::{timeout_add_local, ControlFlow, ExitCode, Propagation},
    prelude::{
        ApplicationExt, ApplicationExtManual, DrawingAreaExtManual, GestureDragExt, GtkWindowExt,
        WidgetExt,
    },
    Application, ApplicationWindow, DrawingArea, EventControllerScroll, EventControllerScrollFlags,
    GestureDrag,
};

use crate::{geometry::FocusRange, view::ViewState};

pub trait Layer {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange);
}

pub struct Visualizer {
    app: Application,
    window_width: i32,
    window_height: i32,
    layers: Vec<(Rc<RefCell<dyn Layer>>, usize)>,
}

impl Visualizer {
    pub fn new(window_width: i32, window_height: i32) -> Self {
        let app = Application::builder()
            .application_id("dev.peruki.visualizer")
            .build();
        Self {
            layers: Vec::new(),
            window_width,
            window_height,
            app,
        }
    }

    pub fn add_layer(&mut self, layer: Rc<RefCell<dyn Layer>>, z_index: usize) {
        self.layers.push((layer, z_index));
    }

    pub fn change_window_size(&mut self, width: i32, height: i32) {
        self.window_width = width;
        self.window_height = height;
    }

    pub fn run(mut self) -> ExitCode {
        self.layers.sort_by_key(|(_, z_index)| *z_index);

        self.app.connect_activate(move |app| {
            let window = ApplicationWindow::builder()
                .application(app)
                .default_width(self.window_width)
                .default_height(self.window_height)
                .title("Visualizer")
                .build();

            let drawing_area = DrawingArea::new();

            let view = Rc::new(RefCell::new(ViewState::new(-2, 2, 10.0)));

            drawing_area.set_draw_func({
                let view = Rc::clone(&view);
                let layers = self
                    .layers
                    .iter()
                    .map(|(layer, _)| Rc::clone(layer))
                    .collect::<Vec<_>>();
                move |drawing_area, cr, _, _| {
                    cr.set_source_rgb(50.0 / 255.0, 110.0 / 255.0, 150.0 / 255.0);
                    cr.paint().expect("Failed to paint background");

                    let mut view = view.borrow_mut();
                    view.update();
                    view.draw_grid(drawing_area, cr);

                    let focus_range = view.focus_range();

                    for layer in &layers {
                        let layer = layer.borrow();
                        layer.draw(drawing_area, cr, focus_range);
                    }
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

            let event_controller_scroll =
                EventControllerScroll::new(EventControllerScrollFlags::all());
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
            timeout_add_local(Duration::from_millis(1000 / 40), tick);
        });

        self.app.run()
    }
}
