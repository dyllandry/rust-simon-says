pub struct TransformComponent {
    pub position: Position,
}

pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl TransformComponent {
    pub fn new() -> Self {
        let position = Position { x: 0.0, y: 0.0 };
        TransformComponent { position }
    }
}
