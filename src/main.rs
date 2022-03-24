// #[macro_use]
extern crate glium;

mod ecs;

fn main() {
    let mut world = ecs::World::new();

    let entity_1 = world.new_entity();
    let name = ecs::Name("Dylan");
    world.add_component_to_entity(entity_1, name);

    let mut names = world.borrow_component_vec::<ecs::Name>().unwrap();
    for name in names.iter_mut().filter_map(|name| name.as_mut()) {
        println!("Name ({})", name.0);
    }

    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    event_loop.run(move |ev, _, control_flow| {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.finish().unwrap();

        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            _ => return,
        }
    });
}
