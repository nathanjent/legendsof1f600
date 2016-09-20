#[macro_use]
extern crate ecs;

use ecs::{World, BuildData, DataHelper, EntityIter, ModifyData, Process, System};
use ecs::system::{EntityProcess, EntitySystem};
use std::io;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Location {
    pub row: i32,
    pub column: i32,
    pub occupant: char,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Velocity {
    pub dx: i32,
    pub dy: i32,
}

pub struct MotionProcess;

impl System for MotionProcess {
    type Components = WorldComponents;
    type Services = ();
}

impl EntityProcess for MotionProcess {
    fn process(&mut self,
               entities: EntityIter<WorldComponents>,
               data: &mut DataHelper<WorldComponents, ()>) {
        for e in entities {
            let mut position = data.position[e];
            let velocity = data.velocity[e];

            position.x += velocity.dx;
            position.y += velocity.dy;
            data.position[e] = position;
        }
    }
}

pub struct Command(pub String);

pub struct CommandProcess;

impl System for CommandProcess {
    type Components = WorldComponents;
    type Services = ();
}

impl EntityProcess for CommandProcess {
    fn process(&mut self,
               entities: EntityIter<WorldComponents>,
               data: &mut DataHelper<WorldComponents, ()>) {
        for e in entities {
        }
    }
}

pub struct RenderView;

impl System for RenderView {
    type Components = WorldComponents;
    type Services = ();
}

impl EntityProcess for RenderView {
    fn process(&mut self,
               entities: EntityIter<WorldComponents>,
               data: &mut DataHelper<WorldComponents, ()>) {
        let view = String::new();
        for e in entities {
        }
        //println!("{}", view);
    }
}

components! {
    struct WorldComponents {
        #[hot] command: Command,
        #[hot] position: Position,
        #[hot] velocity: Velocity,
        #[hot] location: Location,
    }
}

systems! {
    struct WorldSystems<WorldComponents, ()> {
        active: {
            motion: EntitySystem<MotionProcess> = EntitySystem::new(
                MotionProcess,
                aspect!(<WorldComponents> all: [position, velocity])
            ),
            render: EntitySystem<RenderView> = EntitySystem::new(
                RenderView,
                aspect!(<WorldComponents> all: [location])
            ),
        },
        passive: {}
    }
}

fn main() {
    let mut world = World::<WorldSystems>::new();
    let entity = world.create_entity(|entity: BuildData<WorldComponents>,
                                      data: &mut WorldComponents| {
        data.position.add(&entity, Position { x: 0, y: 0 });
        data.velocity.add(&entity, Velocity { dx: 0, dy: 0 });
    });
    let world_map = world.create_entity(|entity: BuildData<WorldComponents>,
                                      data: &mut WorldComponents| {
        data.location.add(&entity, Location { row: 0, column: 0, occupant: '🌲' });
        data.location.add(&entity, Location { row: 0, column: 1, occupant: '🌲' });
        data.location.add(&entity, Location { row: 0, column: 2, occupant: '🌲' });
        data.location.add(&entity, Location { row: 0, column: 3, occupant: '🌳' });
        data.location.add(&entity, Location { row: 0, column: 4, occupant: '🌲' });
        data.location.add(&entity, Location { row: 0, column: 5, occupant: '🌲' });
        data.location.add(&entity, Location { row: 0, column: 6, occupant: '🌲' });
        data.location.add(&entity, Location { row: 0, column: 7, occupant: '🌲' });
        data.location.add(&entity, Location { row: 0, column: 8, occupant: '🌲' });
        data.location.add(&entity, Location { row: 0, column: 9, occupant: '🌲' });
        data.location.add(&entity, Location { row: 1, column: 0, occupant: '🌲' });
        data.location.add(&entity, Location { row: 1, column: 1, occupant: '⬚' });
        data.location.add(&entity, Location { row: 1, column: 2, occupant: '⬚' });
        data.location.add(&entity, Location { row: 1, column: 3, occupant: '⬚' });
        data.location.add(&entity, Location { row: 1, column: 4, occupant: '⬚' });
        data.location.add(&entity, Location { row: 1, column: 5, occupant: '⬚' });
        data.location.add(&entity, Location { row: 1, column: 6, occupant: '⬚' });
        data.location.add(&entity, Location { row: 1, column: 7, occupant: '⬚' });
        data.location.add(&entity, Location { row: 1, column: 8, occupant: '⬚' });
        data.location.add(&entity, Location { row: 1, column: 9, occupant: '🌲' });
        data.location.add(&entity, Location { row: 2, column: 0, occupant: '🌲' });
        data.location.add(&entity, Location { row: 2, column: 1, occupant: '⬚' });
        data.location.add(&entity, Location { row: 2, column: 2, occupant: '⬚' });
        data.location.add(&entity, Location { row: 2, column: 3, occupant: '⬚' });
        data.location.add(&entity, Location { row: 2, column: 4, occupant: '⬚' });
        data.location.add(&entity, Location { row: 2, column: 5, occupant: '⬚' });
        data.location.add(&entity, Location { row: 2, column: 6, occupant: '⬚' });
        data.location.add(&entity, Location { row: 2, column: 7, occupant: '⬚' });
        data.location.add(&entity, Location { row: 2, column: 8, occupant: '⬚' });
        data.location.add(&entity, Location { row: 2, column: 9, occupant: '🌲' });
        data.location.add(&entity, Location { row: 3, column: 0, occupant: '🌲' });
        data.location.add(&entity, Location { row: 3, column: 1, occupant: '⬚' });
        data.location.add(&entity, Location { row: 3, column: 2, occupant: '⬚' });
        data.location.add(&entity, Location { row: 3, column: 3, occupant: '⬚' });
        data.location.add(&entity, Location { row: 3, column: 4, occupant: '⬚' });
        data.location.add(&entity, Location { row: 3, column: 5, occupant: '⬚' });
        data.location.add(&entity, Location { row: 3, column: 6, occupant: '⬚' });
        data.location.add(&entity, Location { row: 3, column: 7, occupant: '⬚' });
        data.location.add(&entity, Location { row: 3, column: 8, occupant: '⬚' });
        data.location.add(&entity, Location { row: 3, column: 9, occupant: '🌲' });
        data.location.add(&entity, Location { row: 4, column: 0, occupant: '🌲' });
        data.location.add(&entity, Location { row: 4, column: 1, occupant: '🌲' });
        data.location.add(&entity, Location { row: 4, column: 2, occupant: '🌲' });
        data.location.add(&entity, Location { row: 4, column: 3, occupant: '🌲' });
        data.location.add(&entity, Location { row: 4, column: 4, occupant: '🌲' });
        data.location.add(&entity, Location { row: 4, column: 5, occupant: '🌲' });
        data.location.add(&entity, Location { row: 4, column: 6, occupant: '🌲' });
        data.location.add(&entity, Location { row: 4, column: 7, occupant: '🌲' });
        data.location.add(&entity, Location { row: 4, column: 8, occupant: '🌲' });
        data.location.add(&entity, Location { row: 4, column: 9, occupant: '🌲' });
    });

    loop {
        let mut input = String::new();
        if let Ok(_) = io::stdin().read_line(&mut input) {
            world.update();
        }
    }
}
