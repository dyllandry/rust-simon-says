pub struct Transform {
    pub position: Position,
}

pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Transform {
    pub fn new() -> Self {
        let position = Position { x: 0.0, y: 0.0 };
        Transform { position }
    }
}
