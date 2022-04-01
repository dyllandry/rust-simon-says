use ecs::{Text, TextSystem};
use glium::{
    glutin::{self, event::Event},
    Surface,
};
mod ecs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::PhysicalSize::new(512, 512))
        .with_title("Rust Simon Says");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut world = ecs::World::new();
    let mut text_system = TextSystem::new(&display);

    let entity = world.new_entity();
    world.add_component_to_entity(
        entity,
        Text {
            text: "Hi Vicky-poo!".to_string(),
        },
    );

    event_loop.run(move |ev, _, control_flow| {
        // Handle events
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => {}
            },

            Event::MainEventsCleared => {
                // Draw background
                let mut frame = display.draw();
                frame.clear_color(1.0, 1.0, 1.0, 0.0);

                // Draw text components
                for text in world
                    .borrow_component_vec::<Text>()
                    .unwrap()
                    .iter()
                    .filter_map(|text| text.as_ref())
                {
                    text_system.draw(&mut frame, &display, text);
                }

                // Finish drawing, swap buffers, consume frame.
                frame.finish().unwrap();
            }
            _ => {}
        }

        // Delay the next loop.
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
    });
}
