#[macro_use]
extern crate ecs;

use ecs::{World, BuildData, DataHelper, EntityIter, ModifyData, Process, System};
use ecs::system::{EntityProcess, EntitySystem};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct WorldMap {
    pub rows: usize,
    pub columns: usize,
    pub buffer: Vec<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Velocity {
    pub dx: u32,
    pub dy: u32,
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
            let map_position = data.world_map[e].buffer.get(
                (position.x + velocity.dx * position.y + velocity.dx) as usize);

            if map_position.unwrap() == &0 {
                position.x += velocity.dx;
                position.y += velocity.dy;
                data.position[e] = position;
            }
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

pub struct PrintMessage(pub String);

impl System for PrintMessage {
    type Components = WorldComponents;
    type Services = ();
}

impl Process for PrintMessage {
    fn process(&mut self, _: &mut DataHelper<WorldComponents, ()>) {
        println!("{}", &self.0);
    }
}

components! {
    struct WorldComponents {
        #[hot] command: Command,
        #[hot] position: Position,
        #[hot] velocity: Velocity,
        #[cold] world_map: WorldMap,
    }
}

systems! {
    struct WorldSystems<WorldComponents, ()> {
        active: {
            motion: EntitySystem<MotionProcess> = EntitySystem::new(
                MotionProcess,
                aspect!(<WorldComponents> all: 
                        [position, velocity, world_map])
            ),
            print_msg: PrintMessage = 
                PrintMessage("Hello World".to_string()),
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
        data.world_map.add(&entity, WorldMap {
            rows: 10,
            columns: 10,
            buffer: vec![
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ] });
    });

    loop {
        let mut input = String::new();
        if let Ok(_) = io::stdin().read_line(&mut input) {
            world.systems.print_msg.0 = format!("Command: {}", input);
            world.update();
        }
    }
}
