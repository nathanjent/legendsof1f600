use specs;
use specs::Join;
use yaml_rust::{Yaml, YamlLoader};

use std::io;

static MAP: &'static str = include_str!("../map.yaml");

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
struct ViewController;

#[derive(Copy, Clone, Debug, PartialEq)]
struct ImmovableController;

impl specs::Component for PlayerController {
    type Storage = specs::VecStorage<PlayerController>;
}

impl specs::Component for NonPlayerController {
    type Storage = specs::VecStorage<NonPlayerController>;
}

impl specs::Component for ViewController {
    type Storage = specs::VecStorage<ViewController>;
}

impl specs::Component for ImmovableController {
    type Storage = specs::VecStorage<ImmovableController>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Position {
    x: i64,
    y: i64,
}

impl specs::Component for Position {
    type Storage = specs::VecStorage<Position>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Velocity {
    dx: i64,
    dy: i64,
}

impl specs::Component for Velocity {
    type Storage = specs::VecStorage<Velocity>;
}


#[derive(Copy, Clone, Debug, PartialEq)]
struct Size {
    w: i64,
    h: i64,
}

impl specs::Component for Size {
    type Storage = specs::VecStorage<Size>;
}

#[derive(Clone, Debug, PartialEq)]
struct WorldMap(Vec<char>);

#[derive(Clone, Debug)]
struct Command(String);
pub fn run() {
    let ref map_cfg = YamlLoader::load_from_str(MAP).unwrap()[0];
    // println!("{:?}", map_cfg);
    let map_width = map_cfg["width"].as_i64().unwrap_or(3);
    let map_height = map_cfg["height"].as_i64().unwrap_or(3);
    let default_map = vec![Yaml::from_str("[
            'T','T', 'T',
            'T','.', 'T',
            'T','T', 'T'
    ]")];
    let map: Vec<char> = map_cfg["world_map"]
                             .as_vec()
                             .unwrap_or(&default_map)
                             .iter()
                             .flat_map(Yaml::as_str)
                             .flat_map(str::chars)
                             .collect();

    let playable: Vec<char> = map_cfg["playable"]
                                  .as_vec()
                                  .unwrap_or(&vec![Yaml::from_str("'X'")])
                                  .iter()
                                  .flat_map(Yaml::as_str)
                                  .flat_map(str::chars)
                                  .collect();

    let blocking: Vec<char> = map_cfg["blocking"]
                                  .as_vec()
                                  .unwrap_or(&vec![Yaml::from_str("'T'")])
                                  .iter()
                                  .flat_map(Yaml::as_str)
                                  .flat_map(str::chars)
                                  .collect();

    let nonblocking: Vec<char> = map_cfg["nonblocking"]
                                     .as_vec()
                                     .unwrap_or(&vec![Yaml::from_str("'.'")])
                                     .iter()
                                     .flat_map(Yaml::as_str)
                                     .flat_map(str::chars)
                                     .collect();

    for c in blocking {
        println!("{}", c);
    }
    let mut planner = {
        let mut w = specs::World::new();
        w.register::<Size>();
        w.register::<Tile>();
        w.register::<Position>();
        w.register::<Velocity>();
        w.register::<PlayerController>();
        w.register::<NonPlayerController>();
        w.register::<ViewController>();
        w.register::<ImmovableController>();

        // Entities
        let _player = w.create_now()
                       .with(Tile { c: 'X' })
                       .with(Velocity { dx: 0, dy: 0 })
                       .with(Position { x: 0, y: 0 })
                       .with(PlayerController)
                       .with(Size { w: 1, h: 1 })
                       .build();

        let _view = w.create_now()
                     .with(Position { x: 0, y: 0 })
                     .with(Size { w: 10, h: 10 })
                     .with(ViewController)
                     .build();

        for (i, tile) in map.iter().enumerate() {
            let _map_tile = w.create_now()
                             .with(ImmovableController)
                             .with(Size { w: 1, h: 1 })
                             .with(Position {
                                 x: i as i64 % map_width,
                                 y: i as i64 / map_width,
                             })
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
        // update positions
        planner.run_custom(|arg| {
            let (mut positions, velocities) = arg.fetch(|w| {
                (w.write::<Position>(), w.read::<Velocity>())
            });

            for (mut p, v) in (&mut positions, &velocities).iter() {
                p.x += v.dx;
                p.y += v.dy;
                println!("{:?}", p);
            }
        });

        planner.wait();

        // update view area
        planner.run_custom(|arg| {
            let (mut positions, players, views, sizes, entities) = arg.fetch(|w| {
                (w.write::<Position>(),
                 w.read::<PlayerController>(),
                 w.read::<ViewController>(),
                 w.read::<Size>(),
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
                                            .expect("view controller expect position component");
                    if x > (size.w / 2) {
                        view_pos.x = x - (size.w / 2);
                    } else {
                        view_pos.x = x;
                    }
                    if y > (size.h / 2) {
                        view_pos.y = y - (size.h / 2);
                    } else {
                        view_pos.y = y;
                    }
                    println!("{:?}", view_pos);
                    println!("{:?}", size);
                }
            }
        });

        planner.wait();

        // render view
        planner.run_custom(|arg| {
            let (tiles, positions, non_players, players, map_objects, views, sizes) =
                arg.fetch(|w| {
                    (w.read::<Tile>(),
                     w.read::<Position>(),
                     w.read::<NonPlayerController>(),
                     w.read::<PlayerController>(),
                     w.read::<ImmovableController>(),
                     w.read::<ViewController>(),
                     w.read::<Size>())
                });
            let mut output = vec![vec![' ';10];10];

            let mut view_state = (None, None, None, None);
            for (_, position, size) in (&views, &positions, &sizes).iter() {
                view_state.0 = Some(position.x);
                view_state.1 = Some(position.y);
                view_state.2 = Some(size.w);
                view_state.3 = Some(size.h);
                break;
            }

            // render in layers
            if let (Some(x), Some(y), Some(width), Some(height)) = view_state {
                for (tile, position, _) in (&tiles, &positions, &map_objects).iter() {
                    if position.x >= x && position.x < x + width {
                        if position.y >= y && position.y < y + height {
                            let _ = output[position.y as usize].remove(position.x as usize);
                            output[position.y as usize].insert(position.x as usize, tile.c);
                        }
                    }
                }
            }
            for (tile, position, _) in (&tiles, &positions, &non_players).iter() {
                let _ = output[position.y as usize].remove(position.x as usize);
                output[position.y as usize].insert(position.x as usize, tile.c);
            }
            for (tile, position, _) in (&tiles, &positions, &players).iter() {
                let _ = output[position.y as usize].remove(position.x as usize);
                output[position.y as usize].insert(position.x as usize, tile.c);
            }
            for out in output {
                let out: String = out.into_iter().collect();
                println!("{}", out);
            }
        });

        // get input
        planner.run_custom(|arg| {
            let mut cmd = arg.fetch(|w| (w.write_resource::<Command>()));
            let mut input = String::new();
            if let Ok(_) = io::stdin().read_line(&mut input) {
                cmd.0 = input;
            }
        });

        planner.wait();

        // process input
        planner.run_custom(|arg| {
            let (mut velocities, players, cmds, entities) = arg.fetch(|w| {
                (w.write::<Velocity>(),
                 w.read::<PlayerController>(),
                 w.read_resource::<Command>(),
                 w.entities())
            });
            let mut commands: Vec<&str> = cmds.0
                                              .split_whitespace()
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
                                dx -= val.parse::<i64>().unwrap_or(1);
                            } else {
                                dx -= 1;
                            }
                        }
                        "right" | "r" => {
                            if let Some(val) = commands.pop() {
                                dx += val.parse::<i64>().unwrap_or(1);
                            } else {
                                dx += 1;
                            }
                        }
                        "up" | "u" => {
                            if let Some(val) = commands.pop() {
                                dy -= val.parse::<i64>().unwrap_or(1);
                            } else {
                                dy -= 1;
                            }
                        }
                        "down" | "d" => {
                            if let Some(val) = commands.pop() {
                                dy += val.parse::<i64>().unwrap_or(1);
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
