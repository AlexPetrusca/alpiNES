pub struct Viewport {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl Viewport {
    pub fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Viewport { x1, y1, x2, y2 }
    }
}