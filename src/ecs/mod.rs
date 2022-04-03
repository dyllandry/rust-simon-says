pub mod component;
pub mod system;
use self::component::Transform;
use std::cell::{RefCell, RefMut};

// World to store component vectors and entity count.
pub struct World {
    entities_count: usize,
    component_vecs: Vec<Box<dyn ComponentVec>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities_count: 0,
            component_vecs: Vec::new(),
        }
    }

    pub fn new_entity(&mut self) -> usize {
        let entity_id = self.entities_count;
        for component_vec in self.component_vecs.iter_mut() {
            component_vec.push_none();
        }
        self.entities_count += 1;
        // Every component gets a transform.
        self.add_component_to_entity(entity_id, Transform::new());
        entity_id
    }

    // ComponentType must be static to support downcasting Any -> ComponentType
    pub fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) {
        // Try to find existing component_vec for ComponentType. Insert component if component_vec
        // is found.
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.get_mut()[entity] = Some(component);
                return;
            }
        }

        // If component_vec not found, create a new vector & insert the component.
        let mut new_component_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.entities_count);
        for _ in 0..self.entities_count {
            new_component_vec.push(None);
        }
        new_component_vec[entity] = Some(component);
        self.component_vecs
            .push(Box::new(RefCell::new(new_component_vec)));
    }

    pub fn borrow_component_vec<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow_mut());
            }
        }
        None
    }

    /// Borrow a specific component from an entity.
    pub fn borrow_component<ComponentType: 'static>(
        &mut self,
        entity: usize,
    ) -> Option<&mut ComponentType> {
        // Try to find the component_vec for ComponentType.
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return component_vec.get_mut()[entity].as_mut();
            }
        }
        None
    }
}

// as_any lets us downcast from ComponentVec -> Any -> concrete component type
trait ComponentVec {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    // Each entity gets an index, all of their components are found at the same index
    // in each component vector. So entity 0's components are found at the 0 index in
    // each component vector. If the entity doesn't have that kind of component, then
    // at that index the vector contains None. Every ComponentVec type must support
    // push_none.
    fn push_none(&mut self);
}

// Casting as Any requires T to be static. Casting as Any supports downcasting
// Any -> concrete component type.
impl<T: 'static> ComponentVec for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
    fn push_none(&mut self) {
        self.get_mut().push(None)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}
