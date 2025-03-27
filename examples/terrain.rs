use gtk4::{cairo::Context, prelude::WidgetExt, DrawingArea};
use std::{cell::RefCell, rc::Rc};
use vislayers::{
    colormap::SimpleColorMap,
    geometry::FocusRange,
    window::{Layer, Visualizer},
};
use worley_particle::map::{
    lerp::{vertorization::Band, InterpolationMethod},
    IDWStrategy, ParticleMap,
};

struct TerrainMap {
    bands: Vec<Band>,
}

impl TerrainMap {
    fn new(file_path: &str, sea_level: f64) -> Self {
        let particle_map =
            ParticleMap::<f64>::read_from_file(file_path).expect("Error reading terrain map");
        let num_thresholds = 80;

        let thresholds = (0..num_thresholds)
            .map(|i| i as f64 * 0.9 / (num_thresholds - 1) as f64 + sea_level)
            .collect::<Vec<_>>();

        let bands = particle_map
            .isobands(
                particle_map.corners(),
                300000.0,
                &thresholds,
                InterpolationMethod::IDW(IDWStrategy::default_from_params(particle_map.params())),
                true,
            )
            .expect("Error generating bands");

        Self { bands }
    }
}

fn bands_step(focus_range: &FocusRange) -> usize {
    (2.0_f64.powi((focus_range.radius() * 8.0).ceil() as i32) as usize).min(16) - 1
}

impl Layer for TerrainMap {
    fn draw(&self, drawing_area: &DrawingArea, cr: &Context, focus_range: &FocusRange) {
        let color_map = SimpleColorMap::new(
            vec![
                [100.0, 150.0, 70.0],
                [60.0, 90.0, 55.0],
                [210.0, 210.0, 210.0],
            ],
            vec![0.0, 0.35, 0.6],
        );

        let area_width = drawing_area.width();
        let area_height = drawing_area.height();

        let rect = focus_range.to_rect(area_width as f64, area_height as f64);

        let bands_step = bands_step(focus_range);

        for threshold in self.bands.iter().step_by(bands_step) {
            cr.new_path();
            for polygon in &threshold.polygons {
                for (i, point) in polygon.iter().enumerate().step_by(bands_step) {
                    let x = rect.map_coord_x(point.0, 0.0, area_width as f64);
                    let y = rect.map_coord_y(point.1, 0.0, area_height as f64);

                    if i == 0 {
                        cr.move_to(x, y);
                    } else {
                        cr.line_to(x, y);
                    }
                }

                cr.close_path();
            }
            let color = color_map.get_color(threshold.threshold);
            cr.set_source_rgb(color[0] / 255.0, color[1] / 255.0, color[2] / 255.0);
            cr.fill().expect("Failed to fill polygon");
        }

        cr.set_source_rgb(1.0, 0.0, 0.0);
        cr.arc(
            rect.map_coord_x(0.0, 0.0, area_width as f64),
            rect.map_coord_y(0.0, 0.0, area_height as f64),
            2.0,
            0.0,
            2.0 * std::f64::consts::PI,
        );
        cr.fill().expect("Failed to draw center point");
    }
}

fn main() {
    let terrain_map = TerrainMap::new("./data/11008264925851530191.particlemap", 0.0025);

    let mut visualizer = Visualizer::new(800, 600);
    visualizer.add_layer(Rc::new(RefCell::new(terrain_map)), 0);
    visualizer.run();
}
