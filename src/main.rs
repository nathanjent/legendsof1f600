#[macro_use]
extern crate ecs;

use ecs::{World, BuildData, DataHelper, EntityIter, ModifyData, Process, System};
use ecs::system::{EntityProcess, EntitySystem};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct WorldMap {
    pub width: usize,
    pub height: usize,
    pub map: Vec<char>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Velocity {
    pub dx: usize,
    pub dy: usize,
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
            let mut velocity = data.velocity[e];
            // only move 1 space per update
            if velocity.dx > 0 {
                if position.x > 0 {
                    position.x += 1;
                }
                velocity.dx -= 1;
            }
            if velocity.dy > 0 {
                if position.y > 0 {
                    position.y += 1;
                }
                velocity.dy -= 1;
            }
            data.position[e] = position;
        }
    }
}

pub struct CommandProcess(pub String);

impl System for CommandProcess {
    type Components = WorldComponents;
    type Services = ();
}

impl EntityProcess for CommandProcess {
    fn process(&mut self, 
               entities: EntityIter<WorldComponents>,
               data: &mut DataHelper<WorldComponents, ()>) {

        let mut commands: Vec<&str> = self.0.split_whitespace()
            .rev().collect();
        let mut dx = 0;
        let mut dy = 0;
        while let Some(word) = commands.pop() {
            match word {
                "left" | "l" => {
                    if let Some(val) = commands.pop() {
                        if dx > 0 {
                            dx -= val.parse::<usize>().unwrap_or(1);
                        }
                    }
                },
                "right" | "r" => {
                    if let Some(val) = commands.pop() {
                        if dx < usize::max_value() {
                            dx += val.parse::<usize>().unwrap_or(1);
                        }
                    }
                },
                "up" | "u" => {
                    if let Some(val) = commands.pop() {
                        if dy < usize::max_value() {
                            dy += val.parse::<usize>().unwrap_or(1);
                        }
                    }
                },
                "down" | "d" => {
                    if let Some(val) = commands.pop() {
                        if dy > 0 {
                            dy -= val.parse::<usize>().unwrap_or(1);
                        }
                    }
                },
                _ => {},
            }
        }
        for e in entities {
            let mut velocity = data.velocity[e];
            velocity.dx += dx;
            velocity.dy += dy;
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
        for e in entities {
            let ref world_map = data.world_map[e];
            let height = world_map.height;
            let width = world_map.width;
            for i in 0..height {
                for j in 0..width {
                    print!("{}", world_map.map[i * width + j]);
                    if j == width - 1 {
                        println!("");
                    }
                }
            }
        }
    }
}

components! {
    struct WorldComponents {
        #[hot] position: Position,
        #[hot] velocity: Velocity,
        #[hot] world_map: WorldMap,
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
                aspect!(<WorldComponents> all: [world_map])
            ),
            command: EntitySystem<CommandProcess> = EntitySystem::new(
                CommandProcess("idle".to_string()),
                aspect!(<WorldComponents> all: [velocity]),
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
        let map_matrix = vec![
            '🌲', '🌲', '🌲', '🌳', '🌲', '🌲', '🌲', '🌲', '🌲', '🌲',
            '🌲',  '_',  '.',  '.',  '.',  '.',  '.',  '.',  '.', '🌲',
            '🌲',  '.',  '.',  '.',  '.',  '.',  '.',  '.',  '.', '🌲',
            '🌲',  '.',  '.',  '.',  '.',  '.',  '.',  '.',  '.', '🌲',
            '🌲',  '.',  '.',  '.',  '.',  '.',  '.',  '.',  '.', '🌲',
            '🌲',  '.',  '.',  '.',  '.',  '🏠',  '.',  '.',  '.', '🌲',
            '🌲',  '.',  '.',  '.',  '.',  '.',  '.',  '.',  '.', '🌲',
            '🌲',  '.',  '.',  '.',  '.',  '.',  '.',  '.',  '.', '🌲',
            '🌲',  '.',  '.',  '.',  '.',  '.',  '.',  '.',  '.', '🌲',
            '🌲', '🌲', '🌲', '🌳', '🌲', '🌲', '🌲', '🌲', '🌲', '🌲'
        ];
        data.world_map.add(&entity, WorldMap { width: 10, height: 10, map: map_matrix });
    });

    loop {
        let mut input = String::new();
        if let Ok(_) = io::stdin().read_line(&mut input) {
            world.systems.command.0 = input;
        }
        world.update();
    }
}
