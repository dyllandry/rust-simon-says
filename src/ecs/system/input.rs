use std::collections::HashMap;

use glium::glutin::event::{VirtualKeyCode, WindowEvent};

use crate::ecs::{component::text::TextComponent, World};

pub struct InputSystem {
    context: Option<Box<dyn Context>>,
}

#[derive(Debug)]
pub struct ProcessedInput {
    keydown: HashMap<VirtualKeyCode, VirtualKeyCode>,
}

impl ProcessedInput {
    pub fn new() -> Self {
        ProcessedInput {
            keydown: HashMap::new(),
        }
    }
}

pub trait Context {
    fn dispatch_input(&self, input: &mut ProcessedInput, world: &mut World);
}

/// This is a sample context that demonstrates how to receive input from the input system and act on the world.
pub struct SampleContext {}

impl Context for SampleContext {
    fn dispatch_input(&self, input: &mut ProcessedInput, world: &mut crate::ecs::World) {
        println!("context got {:?}", input);
        if let Some(text_component_vector) = world.borrow_component_vec::<TextComponent>() {
            for text_component in text_component_vector.iter().filter_map(|t| t.as_ref()) {
                // Here is where you'd do stuff to the components according to the input.
                println!("got text component with text: {}", text_component.text);
            }
        }
    }
}

impl InputSystem {
    pub fn new() -> Self {
        InputSystem { context: None }
    }

    pub fn set_context(&mut self, context: Box<dyn Context>) {
        self.context = Some(context);
    }

    pub fn process_input(&self, event: &WindowEvent, world: &mut World) {
        let mut processed_input = ProcessedInput::new();
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(virtual_keycode) = input.virtual_keycode {
                    processed_input
                        .keydown
                        .insert(virtual_keycode, virtual_keycode);
                }
            }
            _ => {}
        }
        if let Some(context) = self.context.as_ref() {
            context.dispatch_input(&mut processed_input, world);
        }
    }
}
