struct Shape {
    sides: u32,
}

impl Shape {
    fn is_polygon(&self) -> bool {
        self.sides >= 3
    }

    fn label(&self) -> &'static str {
        match self.sides {
            3 => "triangle",
            4 => "quadrilateral",
            _ => "polygon",
        }
    }
}
