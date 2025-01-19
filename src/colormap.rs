pub struct SimpleColorMap {
    colors: Vec<[f32; 3]>,
    thresholds: Vec<f32>,
}

impl SimpleColorMap {
    pub fn new(colors: Vec<[f32; 3]>, thresholds: Vec<f32>) -> Self {
        SimpleColorMap { colors, thresholds }
    }

    pub fn get_color(&self, value: f32) -> [f32; 3] {
        if value < self.thresholds[0] {
            return self.colors[0];
        }

        for i in 1..self.thresholds.len() {
            if value < self.thresholds[i] {
                let ratio = (value - self.thresholds[i - 1])
                    / (self.thresholds[i] - self.thresholds[i - 1]);
                let color = self.colors[i - 1]
                    .iter()
                    .zip(self.colors[i].iter())
                    .map(|(&c1, &c2)| c1 + ratio * (c2 - c1))
                    .collect::<Vec<f32>>();
                return [color[0], color[1], color[2]];
            }
        }

        self.colors[self.colors.len() - 1]
    }
}
