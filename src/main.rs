extern crate specs;

use specs::Join;
use std::io::prelude::*;
use std::io;
use std::io::BufReader;
use std::fs::File;

static MAP: &'static str = include_str!("../map");

#[derive(Copy, Clone, Debug, PartialEq)]
struct Tile {
    c: char,
}

impl specs::Component for Tile {
    type Storage = specs::VecStorage<Tile>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct PlayerController;

#[derive(Copy, Clone, Debug, PartialEq)]
struct NonPlayerController;

#[derive(Copy, Clone, Debug, PartialEq)]
struct RenderController;

#[derive(Copy, Clone, Debug, PartialEq)]
struct ImmovableController;

impl specs::Component for PlayerController {
    type Storage = specs::VecStorage<PlayerController >;
}

impl specs::Component for NonPlayerController {
    type Storage = specs::VecStorage<NonPlayerController >;
}

impl specs::Component for RenderController {
    type Storage = specs::VecStorage<RenderController >;
}

impl specs::Component for ImmovableController {
    type Storage = specs::VecStorage<ImmovableController>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

impl specs::Component for Position {
    type Storage = specs::VecStorage<Position>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Velocity {
    dx: i32,
    dy: i32,
}

impl specs::Component for Velocity {
    type Storage = specs::VecStorage<Velocity>;
}


#[derive(Copy, Clone, Debug, PartialEq)]
struct Size {
    w: u32,
    h: u32,
}

impl specs::Component for Size {
    type Storage = specs::VecStorage<Size>;
}

#[derive(Clone, Debug, PartialEq)]
struct WorldMap(Vec<char>);

#[derive(Clone, Debug)]
struct Command(String);

//#[cfg(not(feature="parallel"))]
//fn main() {
//}
//
//#[cfg(feature="parallel")]
fn main() {
    let map_width = 10;
    let height = 10;
    let mut lines: Vec<&str> = MAP.lines().collect();
    let map_settings = lines.remove(0);
    for word in map_settings.split_whitespace() {
    }
    let map: Vec<char> = lines.iter().flat_map(|line| line.chars()).collect();

    let mut planner = {
        let mut w = specs::World::new();
        w.register::<Size>();
        w.register::<Tile>();
        w.register::<Position>();
        w.register::<Velocity>();
        w.register::<PlayerController>();
        w.register::<NonPlayerController>();
        w.register::<RenderController>();
        w.register::<ImmovableController>();

        // Entities
        let _player = w.create_now()
            .with(Tile { c: '😀' })
            .with(Velocity { dx: 0, dy: 0 })
            .with(Position { x: 0, y: 0 })
            .with(PlayerController)
            .with(Size { w: 1, h: 1 })
            .build();

        let _view = w.create_now()
            .with(Position { x: 0, y: 0 })
            .with(Size { w: 10, h: 10 })
            .with(RenderController)
            .build();

        for (i, tile) in map.iter().enumerate() {
            let _map_tile = w.create_now()
                .with(ImmovableController)
                .with(Size { w: 1, h: 1 })
                .with(Position { x: i as i32 % map_width, y: i as i32 / map_width, })
                .with(Tile { c: tile.clone() })
                .build();
        }

        // resources can be installed, these are nothing fancy, but allow you
        // to pass data to systems and follow the same sync strategy as the
        // component storage does.
        w.add_resource(Command(String::new()));

        // Planner is used to run systems on the specified world with a specified number of threads
        (specs::Planner::<()>::new(w, 4))
    };
    loop {
        planner.run_custom(|arg| {
            let (mut positions, velocities) = arg.fetch(|w| {
                (w.write::<Position>(), w.read::<Velocity>())
            });

            for (mut p, v) in (&mut positions, &velocities).iter() {
                p.x += v.dx;
                p.y += v.dy;
            }
        });

        planner.wait();

        // render view
        planner.run_custom(|arg| {
            let (tiles, positions) = arg.fetch(|w| {
                (w.read::<Tile>(), w.read::<Position>())
            });
            let mut output = Vec::with_capacity(100);

            for (tile, position) in (&tiles, &positions).iter() {
                println!("{:?} {:?}", tile, position);
                output.insert((position.y * 10 + position.x) as usize, tile.c); 
            }
            //for x in 0..m.width {
            //    for y in 0..m.height {
            //        if x == p.x as i32 && y == p.y as i32 {
            //        } else {
            //            print!("{}", world_map.map[i * width + j]);
            //        }
            //        if j == width - 1 {
            //            println!("");
            //        }
            //    }
            //}
            let output: String = output.into_iter().collect();
            println!("{}", output);
        });

        planner.wait();

        // set view
        planner.run_custom(|arg| {
            let (mut positions, players, views, map_objects, sizes, entities) = arg.fetch(|w| {
                (w.write::<Position>(), w.read::<PlayerController>(), w.read::<RenderController>(),
                w.read::<ImmovableController>(), w.read::<Size>(),
                w.entities())
            });

            let mut pos = (None, None);
            for (_, position) in (&players, &positions).iter() {
                pos.0 = Some(position.x);
                pos.1 = Some(position.y);
                break;
            }

            if let (Some(x), Some(y)) = pos {
                for (_, size, entity) in (&views, &sizes, &entities).iter() {
                    let view_pos = positions.get_mut(entity)
                        .expect("render controller expect position component");
                    view_pos.x = x - (size.w as i32 / 2);
                    view_pos.y = y - (size.h as i32 / 2);
                }
            }
        });
            
        // get input
        planner.run_custom(|arg| {
            let mut cmd = arg.fetch(|w| {
                (w.write_resource::<Command>())
            });
            let mut input = String::new();
            if let Ok(_) = io::stdin().read_line(&mut input) {
                cmd.0 = input;
            }
        });

        planner.wait();

        // process input
        planner.run_custom(|arg| {
            let (mut velocities, players, cmds, entities) = arg.fetch(|w| {
                (w.write::<Velocity>(), w.read::<PlayerController>(), 
                 w.read_resource::<Command>(), w.entities())
            });
            let mut commands: Vec<&str> = cmds.0.split_whitespace()
                .rev()
                .collect();

            for (_, entity) in (&players, &entities).iter() {
                let velocity = velocities.get_mut(entity)
                    .expect("player controller expect velocity");
                let mut dx = 0;
                let mut dy = 0;
                while let Some(word) = commands.pop() {
                    match word {
                        "left" | "l" => {
                            if let Some(val) = commands.pop() {
                                dx -= val.parse::<i32>().unwrap_or(1);
                            } else {
                                dx -= 1;
                            }
                        }
                        "right" | "r" => {
                            if let Some(val) = commands.pop() {
                                dx += val.parse::<i32>().unwrap_or(1);
                            } else {
                                dx += 1;
                            }
                        }
                        "up" | "u" => {
                            if let Some(val) = commands.pop() {
                                dy -= val.parse::<i32>().unwrap_or(1);
                            } else {
                                dy -= 1;
                            }
                        }
                        "down" | "d" => {
                            if let Some(val) = commands.pop() {
                                dy += val.parse::<i32>().unwrap_or(1);
                            } else {
                                dy += 1;
                            }
                        }
                        _ => {}
                    }
                }
                velocity.dx += dx;
                velocity.dy += dy;
                println!("{:?}", velocity);
            }
        });

        planner.wait();
    }
}
