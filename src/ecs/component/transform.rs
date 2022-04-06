use glium::Display;

pub struct TransformComponent {
    /// The position of a transform from its anchor.
    pub position: Position,
    pub anchor: Anchor,
    pub width: f32,
}

#[derive(Copy, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

pub enum Anchor {
    TopLeft,
    TopMiddle,
}

impl TransformComponent {
    pub fn new() -> Self {
        let position = Position { x: 0.0, y: 0.0 };
        TransformComponent {
            position,
            width: 100.0,
            anchor: Anchor::TopLeft,
        }
    }

    /// The position of a transform from the top left corner of the screen.
    // I don't think I like having to pass the display as a parameter, I don't think that should
    // be required to know the absolute position.
    pub fn absolute_position(&self, display: &Display) -> Position {
        match self.anchor {
            Anchor::TopMiddle => {
                let window_width = display.gl_window().window().inner_size().width;
                let anchor_position = Position {
                    x: (window_width as f32 / 2.0),
                    y: 0.0,
                };
                Position {
                    x: self.position.x + anchor_position.x - (self.width / 2.0),
                    y: self.position.y + anchor_position.y,
                }
            }
            _ => self.position,
        }
    }
}
