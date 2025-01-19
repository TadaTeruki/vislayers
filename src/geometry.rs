pub struct Rect {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Rect {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn map_coord_x(&self, x: f64, map_min_x: f64, map_max_x: f64) -> f64 {
        (map_min_x + ((x - self.min_x) / self.width()) * (map_max_x - map_min_x))
            .clamp(map_min_x, map_max_x)
    }

    pub fn map_coord_y(&self, y: f64, map_min_y: f64, map_max_y: f64) -> f64 {
        (map_min_y + ((y - self.min_y) / self.height()) * (map_max_y - map_min_y))
            .clamp(map_min_y, map_max_y)
    }
}

#[derive(Debug)]
pub struct FocusRange {
    center_x: f64,
    center_y: f64,
    center_goal_x: f64,
    center_goal_y: f64,
    move_smooth_factor: f64,
    zoom_smooth_factor: f64,
    radius: f64,
    radius_goal: f64,
    move_scale: f64,
}

impl FocusRange {
    pub fn new(center_x: f64, center_y: f64, radius: f64, move_scale: f64) -> Self {
        Self {
            center_x,
            center_y,
            center_goal_x: center_x,
            center_goal_y: center_y,
            move_smooth_factor: 0.5,
            zoom_smooth_factor: 0.5,
            radius,
            radius_goal: radius,
            move_scale,
        }
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    pub fn center(&self) -> (f64, f64) {
        (self.center_x, self.center_y)
    }

    pub fn move_center(&mut self, dx: f64, dy: f64) {
        self.center_goal_x += dx * self.move_scale;
        self.center_goal_y += dy * self.move_scale;
    }

    pub fn set_radius(&mut self, radius: f64) {
        self.radius_goal = radius;
    }

    pub fn update(&mut self) -> bool {
        self.center_x = self.center_x * (1.0 - self.move_smooth_factor)
            + self.center_goal_x * self.move_smooth_factor;
        self.center_y = self.center_y * (1.0 - self.move_smooth_factor)
            + self.center_goal_y * self.move_smooth_factor;
        self.radius = self.radius * (1.0 - self.zoom_smooth_factor)
            + self.radius_goal * self.zoom_smooth_factor;

        true
    }

    pub fn to_rect(&self, image_width: f64, image_height: f64) -> Rect {
        let angle = (image_height / image_width).atan();
        let width_2 = self.radius * angle.cos();
        let height_2 = self.radius * angle.sin();

        Rect::new(
            self.center_x - width_2,
            self.center_y - height_2,
            self.center_x + width_2,
            self.center_y + height_2,
        )
    }
}
