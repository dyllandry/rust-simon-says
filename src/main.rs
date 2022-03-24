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
}
