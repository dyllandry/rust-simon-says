mod ecs;
use ecs::component::text::{TextAlignment, TextComponent};
use ecs::component::transform::{Anchor, TransformComponent};
use ecs::system::input::{InputSystem, SampleContext};
use ecs::system::text::TextSystem;
use glium::{
    glutin::{self, event::Event},
    Surface,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::PhysicalSize::new(512, 512))
        .with_title("Rust Simon Says");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut world = ecs::World::new();
    let mut text_system = TextSystem::new(&display);
    let mut input_system = InputSystem::new();
    let sample_context = SampleContext {};
    input_system.set_context(Box::new(sample_context));

    // Setup title
    let title_entity = world.new_entity();
    world.add_component_to_entity(
        title_entity,
        TextComponent {
            text: "Simon Says".to_string(),
            alignment: TextAlignment::Center,
        },
    );
    let title_transform = world
        .borrow_component::<TransformComponent>(title_entity)
        .unwrap();
    title_transform.width = 300.0;
    title_transform.anchor = Anchor::TopMiddle;
    title_transform.position.y = 20.0;

    // Setup subtitle
    let subtitle_entity = world.new_entity();
    let subtitle_text = TextComponent {
        alignment: TextAlignment::Center,
        text: "Press Enter to Play".to_string(),
    };
    world.add_component_to_entity(subtitle_entity, subtitle_text);
    let subtitle_transform = world
        .borrow_component::<TransformComponent>(subtitle_entity)
        .unwrap();
    // ! FIXME: This width is too small on retina displays. I think I need to do that think where
    // ! you scale by some dpi scale thing. It was in the gpu_cache tutorial for rusttype.
    subtitle_transform.width = 500.0;
    // ! FIXME: I can tell text isn't centered on retina. Didn't test on windows.
    subtitle_transform.anchor = Anchor::TopMiddle;
    subtitle_transform.position.y = 100.0;

    event_loop.run(move |ev, _, control_flow| {
        // Handle events
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => {
                    input_system.process_input(&event, &mut world);
                }
            },

            Event::MainEventsCleared => {
                // Draw background
                let mut frame = display.draw();
                frame.clear_color(1.0, 1.0, 1.0, 0.0);

                // Draw text components
                let texts = world.borrow_component_vec::<TextComponent>().unwrap();
                let transforms = world.borrow_component_vec::<TransformComponent>().unwrap();
                let zip = texts.iter().zip(transforms.iter());
                for (text, transform) in
                    zip.filter_map(|(text, transform)| Some((text.as_ref()?, transform.as_ref()?)))
                {
                    text_system.draw(&mut frame, &display, text, transform);
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
