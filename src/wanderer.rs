use core::cmp::max;

#[derive(Debug)]
pub struct Wanderer {
    start_x: i32,
    start_y: i32,
    dest_x: i32,
    dest_y: i32,
    steps: u32,
    progress: u32,
}

impl Wanderer {
    pub fn new(start_x: i32, start_y: i32, dest_x: i32, dest_y: i32) -> Self {
        let steps = max((dest_x - start_x).abs(), (dest_y - start_y).abs()) as u32;
        let progress = 0;
        Self {
            start_x,
            start_y,
            dest_x,
            dest_y,
            steps,
            progress,
        }
    }
}

impl Iterator for Wanderer {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_x == self.dest_x && self.start_y == self.dest_y {
            return None;
        }
        if self.progress == self.steps {
            return None;
        }
        self.progress += 1;
        let p = self.progress as f32 / self.steps as f32;
        let x = self.start_x + (p * -(self.start_x - self.dest_x) as f32) as i32;
        let y = self.start_y + (p * -(self.start_y - self.dest_y) as f32) as i32;
        Some((x, y))
    }
}
