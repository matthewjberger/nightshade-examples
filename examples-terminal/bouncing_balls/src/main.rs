use nightshade::tui::prelude::*;
use rand::Rng;

const WALL_THICKNESS: u16 = 1;
const BALL_COUNT: usize = 12;
const BALL_SPEED_MIN: f64 = 5.0;
const BALL_SPEED_MAX: f64 = 15.0;

const WALL_LAYER: u32 = 1;
const BALL_LAYER: u32 = 2;

const BALL_CHARACTERS: [char; 6] = ['O', '@', '*', '#', '&', '%'];
const BALL_COLORS: [TermColor; 8] = [
    TermColor::Red,
    TermColor::Green,
    TermColor::Yellow,
    TermColor::Blue,
    TermColor::Magenta,
    TermColor::Cyan,
    TermColor::DarkYellow,
    TermColor::DarkCyan,
];

struct BallData {
    entity: Entity,
    velocity_column: f64,
    velocity_row: f64,
}

struct GameState {
    wall_entities: Vec<Entity>,
    balls: Vec<BallData>,
    hud_entities: EntityGroup,
    arena_left: f64,
    arena_top: f64,
    arena_right: f64,
    arena_bottom: f64,
    collision_count: u64,
}

impl GameState {
    fn new() -> Self {
        Self {
            wall_entities: Vec::new(),
            balls: Vec::new(),
            hud_entities: EntityGroup::new(),
            arena_left: 0.0,
            arena_top: 0.0,
            arena_right: 0.0,
            arena_bottom: 0.0,
            collision_count: 0,
        }
    }

    fn spawn_walls(&mut self, world: &mut World) {
        let terminal = world.resources.terminal_size;
        let columns = terminal.columns;
        let rows = terminal.rows;

        self.arena_left = WALL_THICKNESS as f64;
        self.arena_top = WALL_THICKNESS as f64 + 1.0;
        self.arena_right = (columns - WALL_THICKNESS) as f64;
        self.arena_bottom = (rows - WALL_THICKNESS) as f64;

        let wall_color = TermColor::Grey;
        let wall_background = TermColor::Rgb {
            r: 40,
            g: 40,
            b: 40,
        };

        for column in 0..columns {
            let top_wall = EntityBuilder::new()
                .position(Position {
                    column: column as f64,
                    row: 1.0,
                })
                .sprite(Sprite {
                    character: '═',
                    foreground: wall_color,
                    background: wall_background,
                })
                .collider(Collider {
                    width: 1,
                    height: 1,
                    offset_column: 0.0,
                    offset_row: 0.0,
                    layer: WALL_LAYER,
                    mask: BALL_LAYER,
                })
                .z_index(ZIndex(1))
                .spawn(world);
            self.wall_entities.push(top_wall);

            let bottom_wall = EntityBuilder::new()
                .position(Position {
                    column: column as f64,
                    row: (rows - 1) as f64,
                })
                .sprite(Sprite {
                    character: '═',
                    foreground: wall_color,
                    background: wall_background,
                })
                .collider(Collider {
                    width: 1,
                    height: 1,
                    offset_column: 0.0,
                    offset_row: 0.0,
                    layer: WALL_LAYER,
                    mask: BALL_LAYER,
                })
                .z_index(ZIndex(1))
                .spawn(world);
            self.wall_entities.push(bottom_wall);
        }

        for row in 1..rows {
            let left_wall = EntityBuilder::new()
                .position(Position {
                    column: 0.0,
                    row: row as f64,
                })
                .sprite(Sprite {
                    character: '║',
                    foreground: wall_color,
                    background: wall_background,
                })
                .collider(Collider {
                    width: 1,
                    height: 1,
                    offset_column: 0.0,
                    offset_row: 0.0,
                    layer: WALL_LAYER,
                    mask: BALL_LAYER,
                })
                .z_index(ZIndex(1))
                .spawn(world);
            self.wall_entities.push(left_wall);

            let right_wall = EntityBuilder::new()
                .position(Position {
                    column: (columns - 1) as f64,
                    row: row as f64,
                })
                .sprite(Sprite {
                    character: '║',
                    foreground: wall_color,
                    background: wall_background,
                })
                .collider(Collider {
                    width: 1,
                    height: 1,
                    offset_column: 0.0,
                    offset_row: 0.0,
                    layer: WALL_LAYER,
                    mask: BALL_LAYER,
                })
                .z_index(ZIndex(1))
                .spawn(world);
            self.wall_entities.push(right_wall);
        }
    }

    fn spawn_balls(&mut self, world: &mut World) {
        let mut rng = rand::rng();

        let spawn_left = self.arena_left + 2.0;
        let spawn_right = self.arena_right - 2.0;
        let spawn_top = self.arena_top + 2.0;
        let spawn_bottom = self.arena_bottom - 2.0;

        for ball_index in 0..BALL_COUNT {
            let column = rng.random_range(spawn_left..spawn_right);
            let row = rng.random_range(spawn_top..spawn_bottom);

            let angle = rng.random_range(0.0..std::f64::consts::TAU);
            let speed = rng.random_range(BALL_SPEED_MIN..BALL_SPEED_MAX);
            let velocity_column = angle.cos() * speed;
            let velocity_row = angle.sin() * speed;

            let character = BALL_CHARACTERS[ball_index % BALL_CHARACTERS.len()];
            let color = BALL_COLORS[ball_index % BALL_COLORS.len()];

            let entity = EntityBuilder::new()
                .position(Position { column, row })
                .sprite(Sprite {
                    character,
                    foreground: color,
                    background: TermColor::Black,
                })
                .collider(Collider {
                    width: 1,
                    height: 1,
                    offset_column: 0.0,
                    offset_row: 0.0,
                    layer: BALL_LAYER,
                    mask: WALL_LAYER | BALL_LAYER,
                })
                .z_index(ZIndex(5))
                .spawn(world);

            self.balls.push(BallData {
                entity,
                velocity_column,
                velocity_row,
            });
        }
    }

    fn apply_velocities(&mut self, world: &mut World, delta: f64) {
        for ball in &self.balls {
            if let Some(position) = world.get_position_mut(ball.entity) {
                position.column += ball.velocity_column * delta;
                position.row += ball.velocity_row * delta;
            }
        }
    }

    fn handle_collisions(&mut self, world: &mut World) {
        let contacts = collision_pairs(world);

        for contact in &contacts {
            let is_a_wall = self.wall_entities.contains(&contact.entity_a);
            let is_b_wall = self.wall_entities.contains(&contact.entity_b);

            if is_a_wall || is_b_wall {
                let static_entity = if is_a_wall {
                    contact.entity_a
                } else {
                    contact.entity_b
                };
                let ball_entity = if is_a_wall {
                    contact.entity_b
                } else {
                    contact.entity_a
                };

                resolve_collision_static(world, contact, static_entity);

                if let Some(ball) = self
                    .balls
                    .iter_mut()
                    .find(|ball| ball.entity == ball_entity)
                {
                    let normal_toward_ball = if ball_entity == contact.entity_b {
                        (contact.normal_column, contact.normal_row)
                    } else {
                        (-contact.normal_column, -contact.normal_row)
                    };

                    let dot = ball.velocity_column * normal_toward_ball.0
                        + ball.velocity_row * normal_toward_ball.1;
                    if dot < 0.0 {
                        ball.velocity_column -= 2.0 * dot * normal_toward_ball.0;
                        ball.velocity_row -= 2.0 * dot * normal_toward_ball.1;
                    }
                }

                self.collision_count += 1;
            } else {
                resolve_collision(world, contact);

                let normal_column = contact.normal_column;
                let normal_row = contact.normal_row;

                let velocity_a = self
                    .balls
                    .iter()
                    .find(|ball| ball.entity == contact.entity_a)
                    .map(|ball| (ball.velocity_column, ball.velocity_row));
                let velocity_b = self
                    .balls
                    .iter()
                    .find(|ball| ball.entity == contact.entity_b)
                    .map(|ball| (ball.velocity_column, ball.velocity_row));

                if let (
                    Some((velocity_a_column, velocity_a_row)),
                    Some((velocity_b_column, velocity_b_row)),
                ) = (velocity_a, velocity_b)
                {
                    let relative_column = velocity_a_column - velocity_b_column;
                    let relative_row = velocity_a_row - velocity_b_row;
                    let relative_dot = relative_column * normal_column + relative_row * normal_row;

                    if relative_dot > 0.0 {
                        if let Some(ball_a) = self
                            .balls
                            .iter_mut()
                            .find(|ball| ball.entity == contact.entity_a)
                        {
                            ball_a.velocity_column -= relative_dot * normal_column;
                            ball_a.velocity_row -= relative_dot * normal_row;
                        }
                        if let Some(ball_b) = self
                            .balls
                            .iter_mut()
                            .find(|ball| ball.entity == contact.entity_b)
                        {
                            ball_b.velocity_column += relative_dot * normal_column;
                            ball_b.velocity_row += relative_dot * normal_row;
                        }
                    }
                }

                self.collision_count += 1;
            }
        }
    }

    fn clamp_balls(&mut self, world: &mut World) {
        for ball in &mut self.balls {
            if let Some(position) = world.get_position_mut(ball.entity) {
                if position.column < self.arena_left + 1.0 {
                    position.column = self.arena_left + 1.0;
                    if ball.velocity_column < 0.0 {
                        ball.velocity_column = -ball.velocity_column;
                    }
                }
                if position.column > self.arena_right - 1.0 {
                    position.column = self.arena_right - 1.0;
                    if ball.velocity_column > 0.0 {
                        ball.velocity_column = -ball.velocity_column;
                    }
                }
                if position.row < self.arena_top + 1.0 {
                    position.row = self.arena_top + 1.0;
                    if ball.velocity_row < 0.0 {
                        ball.velocity_row = -ball.velocity_row;
                    }
                }
                if position.row > self.arena_bottom - 1.0 {
                    position.row = self.arena_bottom - 1.0;
                    if ball.velocity_row > 0.0 {
                        ball.velocity_row = -ball.velocity_row;
                    }
                }
            }
        }
    }

    fn update_hud(&mut self, world: &mut World) {
        self.hud_entities.despawn_all(world);

        let hud_text = format!(
            "Bouncing Balls | Balls: {} | Collisions: {} | ESC to quit",
            self.balls.len(),
            self.collision_count,
        );

        let entity = self
            .hud_entities
            .spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 1.0,
                row: 0.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: hud_text,
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
    }
}

impl State for GameState {
    fn title(&self) -> &str {
        "Bouncing Balls"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;
        self.spawn_walls(world);
        self.spawn_balls(world);
        self.update_hud(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        if key == KeyCode::Escape {
            world.resources.should_exit = true;
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;
        self.apply_velocities(world, delta);
        self.handle_collisions(world);
        self.clamp_balls(world);
        self.update_hud(world);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(GameState::new()))
}
