use nightshade::tui::prelude::*;
use rand::Rng;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn delta(self) -> (f64, f64) {
        match self {
            Self::Up => (0.0, -1.0),
            Self::Down => (0.0, 1.0),
            Self::Left => (-1.0, 0.0),
            Self::Right => (1.0, 0.0),
        }
    }
}

struct SnakeGame {
    head_entity: Entity,
    food_entity: Entity,
    body_entities: VecDeque<Entity>,
    border_entities: Vec<Entity>,
    direction: Direction,
    next_direction: Direction,
    move_timer: f64,
    move_interval: f64,
    score: u32,
    game_over: bool,
    play_area_left: i32,
    play_area_top: i32,
    play_area_right: i32,
    play_area_bottom: i32,
    score_entity: Option<Entity>,
    game_over_entities: Vec<Entity>,
}

impl Default for SnakeGame {
    fn default() -> Self {
        Self {
            head_entity: Entity::default(),
            food_entity: Entity::default(),
            body_entities: VecDeque::new(),
            border_entities: Vec::new(),
            direction: Direction::Right,
            next_direction: Direction::Right,
            move_timer: 0.0,
            move_interval: 0.15,
            score: 0,
            game_over: false,
            play_area_left: 1,
            play_area_top: 2,
            play_area_right: 40,
            play_area_bottom: 20,
            score_entity: None,
            game_over_entities: Vec::new(),
        }
    }
}

impl SnakeGame {
    fn spawn_food(&mut self, world: &mut World) {
        let mut rng = rand::rng();

        loop {
            let column = rng.random_range(self.play_area_left..self.play_area_right);
            let row = rng.random_range(self.play_area_top..self.play_area_bottom);

            let head_pos = world.get_position(self.head_entity).unwrap();
            if head_pos.column.round() as i32 == column && head_pos.row.round() as i32 == row {
                continue;
            }

            let on_body = self.body_entities.iter().any(|&body_entity| {
                world.get_position(body_entity).is_some_and(|body_pos| {
                    body_pos.column.round() as i32 == column && body_pos.row.round() as i32 == row
                })
            });
            if on_body {
                continue;
            }

            if let Some(position) = world.get_position_mut(self.food_entity) {
                position.column = column as f64;
                position.row = row as f64;
            }
            break;
        }
    }

    fn spawn_body_segment(&mut self, world: &mut World, column: f64, row: f64) -> Entity {
        let entity = world.spawn_entities(POSITION | SPRITE, 1)[0];
        world.set_position(entity, Position { column, row });
        world.set_sprite(
            entity,
            Sprite {
                character: 'o',
                foreground: TermColor::DarkGreen,
                background: TermColor::Black,
            },
        );
        entity
    }

    fn draw_border(&mut self, world: &mut World) {
        let left = self.play_area_left - 1;
        let right = self.play_area_right;
        let top = self.play_area_top - 1;
        let bottom = self.play_area_bottom;

        for column in left..=right {
            self.spawn_border_cell(world, column, top);
            self.spawn_border_cell(world, column, bottom);
        }

        for row in top..=bottom {
            self.spawn_border_cell(world, left, row);
            self.spawn_border_cell(world, right, row);
        }
    }

    fn spawn_border_cell(&mut self, world: &mut World, column: i32, row: i32) {
        let entity = world.spawn_entities(POSITION | SPRITE, 1)[0];
        world.set_position(
            entity,
            Position {
                column: column as f64,
                row: row as f64,
            },
        );
        world.set_sprite(
            entity,
            Sprite {
                character: '#',
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        self.border_entities.push(entity);
    }

    fn update_score_display(&self, world: &mut World) {
        if let Some(score_entity) = self.score_entity {
            let text = format!("Score: {}", self.score);
            for (index, character) in text.chars().enumerate() {
                let column = self.play_area_left + index as i32;
                let entity_id = format!("score_char_{}", index);

                let existing: Vec<Entity> = world
                    .query_entities(POSITION | SPRITE | NAME)
                    .filter(|&entity| {
                        world
                            .get_name(entity)
                            .is_some_and(|name| name.0 == entity_id)
                    })
                    .collect();

                if let Some(&entity) = existing.first() {
                    if let Some(sprite) = world.get_sprite_mut(entity) {
                        sprite.character = character;
                    }
                    if let Some(position) = world.get_position_mut(entity) {
                        position.column = column as f64;
                    }
                } else {
                    let entity = world.spawn_entities(POSITION | SPRITE | NAME, 1)[0];
                    world.set_position(
                        entity,
                        Position {
                            column: column as f64,
                            row: 0.0,
                        },
                    );
                    world.set_sprite(
                        entity,
                        Sprite {
                            character,
                            foreground: TermColor::Yellow,
                            background: TermColor::Black,
                        },
                    );
                    world.set_name(entity, Name(entity_id));
                }
            }

            if let Some(sprite) = world.get_sprite_mut(score_entity) {
                sprite.character = ' ';
            }
        }
    }

    fn show_game_over(&mut self, world: &mut World) {
        let center_column = (self.play_area_left + self.play_area_right) / 2;
        let center_row = (self.play_area_top + self.play_area_bottom) / 2;

        let line1 = "GAME OVER";
        let line2 = format!("Score: {}", self.score);
        let line3 = "Press R to restart";

        self.draw_text_centered(world, line1, center_column, center_row - 1, TermColor::Red);
        self.draw_text_centered(world, &line2, center_column, center_row, TermColor::Yellow);
        self.draw_text_centered(
            world,
            line3,
            center_column,
            center_row + 1,
            TermColor::White,
        );
    }

    fn draw_text_centered(
        &mut self,
        world: &mut World,
        text: &str,
        center_column: i32,
        row: i32,
        color: TermColor,
    ) {
        let start_column = center_column - (text.len() as i32) / 2;
        for (index, character) in text.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (start_column + index as i32) as f64,
                    row: row as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground: color,
                    background: TermColor::Black,
                },
            );
            self.game_over_entities.push(entity);
        }
    }

    fn restart(&mut self, world: &mut World) {
        for &entity in &self.body_entities {
            world.despawn_entities(&[entity]);
        }
        self.body_entities.clear();

        for &entity in &self.game_over_entities {
            world.despawn_entities(&[entity]);
        }
        self.game_over_entities.clear();

        let score_chars: Vec<Entity> = world
            .query_entities(POSITION | SPRITE | NAME)
            .filter(|&entity| {
                world
                    .get_name(entity)
                    .is_some_and(|name| name.0.starts_with("score_char_"))
            })
            .collect();
        for entity in score_chars {
            world.despawn_entities(&[entity]);
        }

        self.direction = Direction::Right;
        self.next_direction = Direction::Right;
        self.move_timer = 0.0;
        self.move_interval = 0.15;
        self.score = 0;
        self.game_over = false;

        let start_column = (self.play_area_left + self.play_area_right) / 2;
        let start_row = (self.play_area_top + self.play_area_bottom) / 2;

        if let Some(position) = world.get_position_mut(self.head_entity) {
            position.column = start_column as f64;
            position.row = start_row as f64;
        }

        for offset in 1..=3 {
            let segment =
                self.spawn_body_segment(world, (start_column - offset) as f64, start_row as f64);
            self.body_entities.push_back(segment);
        }

        self.spawn_food(world);
        self.update_score_display(world);
    }
}

impl State for SnakeGame {
    fn title(&self) -> &str {
        "Snake - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 60;

        let terminal = world.resources.terminal_size;
        self.play_area_right = (terminal.columns as i32).min(42) - 2;
        self.play_area_bottom = (terminal.rows as i32).min(22) - 2;

        self.draw_border(world);

        let start_column = (self.play_area_left + self.play_area_right) / 2;
        let start_row = (self.play_area_top + self.play_area_bottom) / 2;

        self.head_entity = world.spawn_entities(POSITION | SPRITE, 1)[0];
        world.set_position(
            self.head_entity,
            Position {
                column: start_column as f64,
                row: start_row as f64,
            },
        );
        world.set_sprite(
            self.head_entity,
            Sprite {
                character: '@',
                foreground: TermColor::Green,
                background: TermColor::Black,
            },
        );

        for offset in 1..=3 {
            let segment =
                self.spawn_body_segment(world, (start_column - offset) as f64, start_row as f64);
            self.body_entities.push_back(segment);
        }

        self.food_entity = world.spawn_entities(POSITION | SPRITE, 1)[0];
        world.set_sprite(
            self.food_entity,
            Sprite {
                character: '*',
                foreground: TermColor::Red,
                background: TermColor::Black,
            },
        );
        self.spawn_food(world);

        self.score_entity = Some(world.spawn_entities(POSITION | SPRITE, 1)[0]);
        self.update_score_display(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }

        match key {
            KeyCode::Escape | KeyCode::Char('q') => {
                world.resources.should_exit = true;
            }
            KeyCode::Char('r') if self.game_over => {
                self.restart(world);
            }
            KeyCode::Up | KeyCode::Char('w') if !self.game_over => {
                if self.direction != Direction::Down {
                    self.next_direction = Direction::Up;
                }
            }
            KeyCode::Down | KeyCode::Char('s') if !self.game_over => {
                if self.direction != Direction::Up {
                    self.next_direction = Direction::Down;
                }
            }
            KeyCode::Left | KeyCode::Char('a') if !self.game_over => {
                if self.direction != Direction::Right {
                    self.next_direction = Direction::Left;
                }
            }
            KeyCode::Right | KeyCode::Char('d') if !self.game_over => {
                if self.direction != Direction::Left {
                    self.next_direction = Direction::Right;
                }
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        if self.game_over {
            return;
        }

        self.move_timer += world.resources.timing.delta_seconds;

        if self.move_timer < self.move_interval {
            return;
        }
        self.move_timer = 0.0;

        self.direction = self.next_direction;
        let (delta_column, delta_row) = self.direction.delta();

        let head_pos = *world.get_position(self.head_entity).unwrap();
        let new_column = head_pos.column + delta_column;
        let new_row = head_pos.row + delta_row;
        let new_column_i32 = new_column.round() as i32;
        let new_row_i32 = new_row.round() as i32;

        if new_column_i32 < self.play_area_left
            || new_column_i32 >= self.play_area_right
            || new_row_i32 < self.play_area_top
            || new_row_i32 >= self.play_area_bottom
        {
            self.game_over = true;
            self.show_game_over(world);
            return;
        }

        for &body_entity in &self.body_entities {
            if let Some(body_pos) = world.get_position(body_entity)
                && body_pos.column.round() as i32 == new_column_i32
                && body_pos.row.round() as i32 == new_row_i32
            {
                self.game_over = true;
                self.show_game_over(world);
                return;
            }
        }

        let new_segment = self.spawn_body_segment(world, head_pos.column, head_pos.row);
        self.body_entities.push_front(new_segment);

        if let Some(position) = world.get_position_mut(self.head_entity) {
            position.column = new_column;
            position.row = new_row;
        }

        let food_pos = *world.get_position(self.food_entity).unwrap();
        if new_column_i32 == food_pos.column.round() as i32
            && new_row_i32 == food_pos.row.round() as i32
        {
            self.score += 1;
            self.move_interval = (self.move_interval * 0.95).max(0.05);
            self.spawn_food(world);
            self.update_score_display(world);
        } else if let Some(tail) = self.body_entities.pop_back() {
            world.despawn_entities(&[tail]);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(SnakeGame::default()))
}
