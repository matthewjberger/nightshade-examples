use nightshade::tui::prelude::*;
use rand::Rng;

const STARTING_PLAYER_HP: i32 = 80;
const STARTING_PLAYER_MAX_HP: i32 = 80;
const HAND_SIZE: usize = 5;
const ENERGY_PER_TURN: i32 = 3;
const TOTAL_REGULAR_FIGHTS: usize = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CardType {
    Strike,
    Defend,
    HeavyStrike,
    DoubleDefend,
    Fireball,
}

#[derive(Clone)]
struct Card {
    card_type: CardType,
    name: String,
    description: String,
    energy_cost: i32,
}

fn make_card(card_type: CardType) -> Card {
    match card_type {
        CardType::Strike => Card {
            card_type,
            name: "Strike".to_string(),
            description: "Deal 6 damage".to_string(),
            energy_cost: 1,
        },
        CardType::Defend => Card {
            card_type,
            name: "Defend".to_string(),
            description: "Gain 5 block".to_string(),
            energy_cost: 1,
        },
        CardType::HeavyStrike => Card {
            card_type,
            name: "Heavy Strike".to_string(),
            description: "Deal 12 damage".to_string(),
            energy_cost: 2,
        },
        CardType::DoubleDefend => Card {
            card_type,
            name: "Double Defend".to_string(),
            description: "Gain 10 block".to_string(),
            energy_cost: 2,
        },
        CardType::Fireball => Card {
            card_type,
            name: "Fireball".to_string(),
            description: "Deal 8 to all".to_string(),
            energy_cost: 2,
        },
    }
}

fn starting_deck() -> Vec<Card> {
    let mut deck = Vec::new();
    for _ in 0..5 {
        deck.push(make_card(CardType::Strike));
    }
    for _ in 0..5 {
        deck.push(make_card(CardType::Defend));
    }
    deck
}

fn reward_card_pool() -> Vec<CardType> {
    vec![
        CardType::Strike,
        CardType::Defend,
        CardType::HeavyStrike,
        CardType::DoubleDefend,
        CardType::Fireball,
    ]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnemyIntent {
    Attack(i32),
    Defend(i32),
}

#[derive(Clone)]
struct EnemyData {
    name: String,
    hp: i32,
    max_hp: i32,
    intent: EnemyIntent,
    block: i32,
}

fn generate_regular_enemies(fight_number: usize) -> Vec<EnemyData> {
    let mut rng = rand::rng();
    let enemy_count = if fight_number == 0 {
        2
    } else {
        2 + rng.random_range(0..2)
    };
    let mut enemies = Vec::new();
    let base_hp = 20 + fight_number as i32 * 8;
    let base_attack = 5 + fight_number as i32 * 2;

    let names = ["Slime", "Goblin", "Cultist", "Bandit", "Fungus"];

    for index in 0..enemy_count {
        let name_index = rng.random_range(0..names.len());
        let hp_variance = rng.random_range(-3..=5);
        let hp = base_hp + hp_variance;
        let attack = base_attack + rng.random_range(0..=2);
        enemies.push(EnemyData {
            name: format!("{} {}", names[name_index], (b'A' + index as u8) as char),
            hp,
            max_hp: hp,
            intent: EnemyIntent::Attack(attack),
            block: 0,
        });
    }
    enemies
}

fn generate_boss() -> Vec<EnemyData> {
    vec![EnemyData {
        name: "The Guardian".to_string(),
        hp: 100,
        max_hp: 100,
        intent: EnemyIntent::Attack(15),
        block: 0,
    }]
}

fn roll_enemy_intent(enemy: &mut EnemyData) {
    let mut rng = rand::rng();
    let roll: f64 = rng.random();
    if roll < 0.6 {
        let base_attack = 5 + (enemy.max_hp / 10);
        let variance = rng.random_range(-1..=2);
        enemy.intent = EnemyIntent::Attack((base_attack + variance).max(1));
    } else {
        let base_defend = 4 + (enemy.max_hp / 15);
        let variance = rng.random_range(0..=2);
        enemy.intent = EnemyIntent::Defend(base_defend + variance);
    }
}

fn card_border_color(card_type: CardType) -> TermColor {
    match card_type {
        CardType::Strike => TermColor::Red,
        CardType::Defend => TermColor::Blue,
        CardType::HeavyStrike => TermColor::DarkRed,
        CardType::DoubleDefend => TermColor::DarkBlue,
        CardType::Fireball => TermColor::Yellow,
    }
}

fn card_width() -> usize {
    18
}

fn card_height() -> usize {
    7
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Deck Builder - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "DECK BUILDER";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - title.len() as f64 / 2.0,
                row: center_row - 5.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: title.to_string(),
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let subtitle = "A Slay the Spire-lite Card Game";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - subtitle.len() as f64 / 2.0,
                row: center_row - 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: subtitle.to_string(),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let line1 = "Arrow Keys: select card    Enter: play card";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - line1.len() as f64 / 2.0,
                row: center_row - 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: line1.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let line2 = "E: end turn    ESC: quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - line2.len() as f64 / 2.0,
                row: center_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: line2.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let prompt = "Press ENTER to begin";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - prompt.len() as f64 / 2.0,
                row: center_row + 2.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: prompt.to_string(),
                foreground: TermColor::Green,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let quit = "ESC to quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - quit.len() as f64 / 2.0,
                row: center_row + 4.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: quit.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Enter => self.start_game = true,
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.start_game {
            self.entities.despawn_all(world);
            return Some(Box::new(CombatState::new(
                starting_deck(),
                STARTING_PLAYER_HP,
                STARTING_PLAYER_MAX_HP,
                0,
                false,
            )));
        }
        None
    }
}

struct CombatState {
    deck: Vec<Card>,
    draw_pile: Vec<Card>,
    discard_pile: Vec<Card>,
    hand: Vec<Card>,
    enemies: Vec<EnemyData>,
    player_hp: i32,
    player_max_hp: i32,
    player_block: i32,
    energy: i32,
    max_energy: i32,
    selected_card_index: usize,
    fight_number: usize,
    is_boss: bool,
    combat_log: Vec<String>,
    transition: CombatTransition,
    entities: EntityGroup,
    particles: ParticleEmitter,
    turn_number: i32,
}

#[derive(PartialEq, Eq)]
enum CombatTransition {
    None,
    Victory,
    Defeat,
}

impl CombatState {
    fn new(
        deck: Vec<Card>,
        player_hp: i32,
        player_max_hp: i32,
        fight_number: usize,
        is_boss: bool,
    ) -> Self {
        let enemies = if is_boss {
            generate_boss()
        } else {
            generate_regular_enemies(fight_number)
        };

        Self {
            deck: deck.clone(),
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            hand: Vec::new(),
            enemies,
            player_hp,
            player_max_hp,
            player_block: 0,
            energy: ENERGY_PER_TURN,
            max_energy: ENERGY_PER_TURN,
            selected_card_index: 0,
            fight_number,
            is_boss,
            combat_log: Vec::new(),
            transition: CombatTransition::None,
            entities: EntityGroup::new(),
            particles: ParticleEmitter::new(),
            turn_number: 0,
        }
    }

    fn start_combat(&mut self) {
        self.draw_pile = self.deck.clone();
        self.shuffle_draw_pile();
        self.discard_pile.clear();
        self.hand.clear();
        self.player_block = 0;
        self.turn_number = 0;
        self.start_turn();
    }

    fn shuffle_draw_pile(&mut self) {
        let mut rng = rand::rng();
        let length = self.draw_pile.len();
        for index in (1..length).rev() {
            let swap_index = rng.random_range(0..=index);
            self.draw_pile.swap(index, swap_index);
        }
    }

    fn start_turn(&mut self) {
        self.turn_number += 1;
        self.energy = self.max_energy;
        self.player_block = 0;
        self.draw_cards(HAND_SIZE);
        if self.selected_card_index >= self.hand.len() && !self.hand.is_empty() {
            self.selected_card_index = 0;
        }
        self.add_log(format!("--- Turn {} ---", self.turn_number));
    }

    fn draw_cards(&mut self, count: usize) {
        for _ in 0..count {
            if self.draw_pile.is_empty() {
                if self.discard_pile.is_empty() {
                    break;
                }
                self.draw_pile.append(&mut self.discard_pile);
                self.shuffle_draw_pile();
            }
            if let Some(card) = self.draw_pile.pop() {
                self.hand.push(card);
            }
        }
    }

    fn play_selected_card(&mut self, world: &mut World) {
        if self.hand.is_empty() {
            return;
        }
        if self.selected_card_index >= self.hand.len() {
            return;
        }

        let card = &self.hand[self.selected_card_index];
        if card.energy_cost > self.energy {
            self.add_log("Not enough energy!".to_string());
            return;
        }

        if self.enemies.is_empty() {
            return;
        }

        let card_type = card.card_type;
        let card_name = card.name.clone();
        let cost = card.energy_cost;

        self.energy -= cost;

        match card_type {
            CardType::Strike => {
                let target_index = self.find_first_alive_enemy();
                if let Some(target) = target_index {
                    let damage = self.apply_damage_to_enemy(target, 6);
                    self.add_log(format!(
                        "{} deals {} damage to {}",
                        card_name, damage, self.enemies[target].name
                    ));
                    self.emit_attack_particles(world, target);
                }
            }
            CardType::Defend => {
                self.player_block += 5;
                self.add_log(format!("{} grants 5 block", card_name));
            }
            CardType::HeavyStrike => {
                let target_index = self.find_first_alive_enemy();
                if let Some(target) = target_index {
                    let damage = self.apply_damage_to_enemy(target, 12);
                    self.add_log(format!(
                        "{} deals {} damage to {}",
                        card_name, damage, self.enemies[target].name
                    ));
                    self.emit_attack_particles(world, target);
                }
            }
            CardType::DoubleDefend => {
                self.player_block += 10;
                self.add_log(format!("{} grants 10 block", card_name));
            }
            CardType::Fireball => {
                let mut messages = Vec::new();
                for enemy_index in 0..self.enemies.len() {
                    let damage = self.apply_damage_to_enemy(enemy_index, 8);
                    messages.push(format!(
                        "  {} takes {} damage",
                        self.enemies[enemy_index].name, damage
                    ));
                }
                self.add_log(format!("{} hits all enemies!", card_name));
                for message in messages {
                    self.add_log(message);
                }
                self.emit_fireball_particles(world);
            }
        }

        let played_card = self.hand.remove(self.selected_card_index);
        self.discard_pile.push(played_card);

        if self.selected_card_index >= self.hand.len() && !self.hand.is_empty() {
            self.selected_card_index = self.hand.len() - 1;
        }

        self.remove_dead_enemies();
    }

    fn find_first_alive_enemy(&self) -> Option<usize> {
        self.enemies.iter().position(|enemy| enemy.hp > 0)
    }

    fn apply_damage_to_enemy(&mut self, enemy_index: usize, raw_damage: i32) -> i32 {
        let enemy = &mut self.enemies[enemy_index];
        let blocked = raw_damage.min(enemy.block);
        enemy.block -= blocked;
        let remaining_damage = raw_damage - blocked;
        enemy.hp -= remaining_damage;
        remaining_damage
    }

    fn remove_dead_enemies(&mut self) {
        let mut index = 0;
        while index < self.enemies.len() {
            if self.enemies[index].hp <= 0 {
                let dead_name = self.enemies[index].name.clone();
                self.add_log(format!("{} is defeated!", dead_name));
                self.enemies.remove(index);
            } else {
                index += 1;
            }
        }

        if self.enemies.is_empty() {
            self.transition = CombatTransition::Victory;
        }
    }

    fn end_turn(&mut self) {
        self.discard_pile.append(&mut self.hand);
        self.selected_card_index = 0;
        self.enemy_turn();

        if self.player_hp <= 0 {
            self.transition = CombatTransition::Defeat;
            return;
        }

        self.remove_dead_enemies();
        if self.transition == CombatTransition::Victory {
            return;
        }

        for enemy in &mut self.enemies {
            roll_enemy_intent(enemy);
        }

        self.start_turn();
    }

    fn enemy_turn(&mut self) {
        self.add_log("--- Enemy Turn ---".to_string());

        for enemy_index in 0..self.enemies.len() {
            let enemy = &mut self.enemies[enemy_index];
            enemy.block = 0;

            match enemy.intent {
                EnemyIntent::Attack(damage) => {
                    let enemy_name = enemy.name.clone();
                    let blocked = damage.min(self.player_block);
                    self.player_block -= blocked;
                    let actual_damage = damage - blocked;
                    self.player_hp -= actual_damage;
                    if blocked > 0 {
                        self.add_log(format!(
                            "{} attacks for {} (blocked {})",
                            enemy_name, actual_damage, blocked
                        ));
                    } else {
                        self.add_log(format!("{} attacks for {}", enemy_name, actual_damage));
                    }
                }
                EnemyIntent::Defend(block) => {
                    let enemy_name = enemy.name.clone();
                    self.enemies[enemy_index].block += block;
                    self.add_log(format!("{} gains {} block", enemy_name, block));
                }
            }
        }
    }

    fn emit_attack_particles(&mut self, world: &mut World, enemy_index: usize) {
        let terminal = world.resources.terminal_size;
        let enemy_column = self.enemy_display_column(terminal.columns, enemy_index);
        let enemy_row = 4.0;

        self.particles.emit(
            world,
            enemy_column,
            enemy_row,
            6,
            &ParticleConfig {
                characters: vec!['*', '+', '!'],
                colors: vec![TermColor::Red, TermColor::Yellow, TermColor::White],
                lifetime: 0.4,
                speed_min: 2.0,
                speed_max: 5.0,
                spread: std::f64::consts::PI * 2.0,
                direction: 0.0,
                z_index: 20,
            },
        );
    }

    fn emit_fireball_particles(&mut self, world: &mut World) {
        let terminal = world.resources.terminal_size;
        for enemy_index in 0..self.enemies.len() {
            let enemy_column = self.enemy_display_column(terminal.columns, enemy_index);
            let enemy_row = 4.0;
            self.particles.emit(
                world,
                enemy_column,
                enemy_row,
                4,
                &ParticleConfig {
                    characters: vec!['*', '~', '#'],
                    colors: vec![TermColor::Yellow, TermColor::Red, TermColor::DarkYellow],
                    lifetime: 0.5,
                    speed_min: 1.5,
                    speed_max: 4.0,
                    spread: std::f64::consts::PI * 2.0,
                    direction: 0.0,
                    z_index: 20,
                },
            );
        }
    }

    fn enemy_display_column(&self, terminal_width: u16, enemy_index: usize) -> f64 {
        let total_enemies = self.enemies.len().max(1);
        let spacing = terminal_width as f64 / (total_enemies as f64 + 1.0);
        spacing * (enemy_index as f64 + 1.0)
    }

    fn add_log(&mut self, message: String) {
        self.combat_log.push(message);
        if self.combat_log.len() > 100 {
            self.combat_log.remove(0);
        }
    }

    fn render_all(&mut self, world: &mut World) {
        self.entities.despawn_all(world);

        let terminal = world.resources.terminal_size;
        let screen_width = terminal.columns as f64;
        let screen_height = terminal.rows as f64;

        self.render_header(world, screen_width);
        self.render_enemies(world, terminal.columns);
        self.render_hand(world, screen_width, screen_height);
        self.render_log(world, screen_width, screen_height);
    }

    fn render_header(&mut self, world: &mut World, screen_width: f64) {
        let fight_label = if self.is_boss {
            "BOSS FIGHT".to_string()
        } else {
            format!("Fight {}/{}", self.fight_number + 1, TOTAL_REGULAR_FIGHTS)
        };

        let header = format!(
            "HP: {}/{}    Block: {}    Energy: {}/{}    {}",
            self.player_hp,
            self.player_max_hp,
            self.player_block,
            self.energy,
            self.max_energy,
            fight_label,
        );

        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
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
                text: header,
                foreground: TermColor::White,
                background: TermColor::Rgb {
                    r: 20,
                    g: 20,
                    b: 40,
                },
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let bar_width = screen_width as usize - 2;
        let hp_fraction = self.player_hp as f64 / self.player_max_hp as f64;
        let filled_cells = (hp_fraction * bar_width as f64).round() as usize;
        let mut bar_text = String::new();
        for cell_index in 0..bar_width {
            if cell_index < filled_cells {
                bar_text.push('█');
            } else {
                bar_text.push('░');
            }
        }
        let hp_color = if hp_fraction > 0.6 {
            TermColor::Green
        } else if hp_fraction > 0.3 {
            TermColor::Yellow
        } else {
            TermColor::Red
        };

        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: 1.0,
                row: 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: bar_text,
                foreground: hp_color,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
    }

    fn render_enemies(&mut self, world: &mut World, terminal_width: u16) {
        for (enemy_index, enemy) in self.enemies.iter().enumerate() {
            let center_column = self.enemy_display_column(terminal_width, enemy_index);

            let name_text = enemy.name.to_string();
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - name_text.len() as f64 / 2.0,
                    row: 3.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: name_text,
                    foreground: TermColor::Red,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));

            let enemy_art = if self.is_boss {
                vec![
                    r"  /\_/\  ",
                    r" ( o.o ) ",
                    r"  > ^ <  ",
                    r" /|   |\ ",
                    r"(_|   |_)",
                ]
            } else {
                vec![r"  /\ ", r" (oo)", r" /||\", r"  /\ "]
            };

            for (line_index, line) in enemy_art.iter().enumerate() {
                let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: center_column - line.len() as f64 / 2.0,
                        row: 4.0 + line_index as f64,
                    },
                );
                let enemy_color = if self.is_boss {
                    TermColor::Magenta
                } else {
                    TermColor::DarkRed
                };
                world.set_label(
                    entity,
                    Label {
                        text: line.to_string(),
                        foreground: enemy_color,
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(10));
            }

            let art_bottom = 4.0 + enemy_art.len() as f64;

            let hp_text = format!("HP: {}/{}", enemy.hp.max(0), enemy.max_hp);
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - hp_text.len() as f64 / 2.0,
                    row: art_bottom,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: hp_text,
                    foreground: TermColor::White,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));

            if enemy.block > 0 {
                let block_text = format!("Block: {}", enemy.block);
                let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: center_column - block_text.len() as f64 / 2.0,
                        row: art_bottom + 1.0,
                    },
                );
                world.set_label(
                    entity,
                    Label {
                        text: block_text,
                        foreground: TermColor::Cyan,
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(10));
            }

            let intent_row = art_bottom + if enemy.block > 0 { 2.0 } else { 1.0 };
            let intent_text = match enemy.intent {
                EnemyIntent::Attack(damage) => format!("Intent: Attack {}", damage),
                EnemyIntent::Defend(block) => format!("Intent: Defend {}", block),
            };
            let intent_color = match enemy.intent {
                EnemyIntent::Attack(_) => TermColor::Red,
                EnemyIntent::Defend(_) => TermColor::Blue,
            };
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - intent_text.len() as f64 / 2.0,
                    row: intent_row,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: intent_text,
                    foreground: intent_color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));

            let hp_bar_width = 12;
            let hp_fraction = enemy.hp.max(0) as f64 / enemy.max_hp as f64;
            let filled = (hp_fraction * hp_bar_width as f64).round() as usize;
            let mut hp_bar = String::new();
            for bar_index in 0..hp_bar_width {
                if bar_index < filled {
                    hp_bar.push('█');
                } else {
                    hp_bar.push('░');
                }
            }
            let bar_color = if hp_fraction > 0.6 {
                TermColor::Green
            } else if hp_fraction > 0.3 {
                TermColor::Yellow
            } else {
                TermColor::Red
            };
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - hp_bar_width as f64 / 2.0,
                    row: intent_row + 1.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: hp_bar,
                    foreground: bar_color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }
    }

    fn render_hand(&mut self, world: &mut World, screen_width: f64, screen_height: f64) {
        if self.hand.is_empty() {
            let empty_text = "No cards in hand - press E to end turn";
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: screen_width / 2.0 - empty_text.len() as f64 / 2.0,
                    row: screen_height - card_height() as f64 - 3.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: empty_text.to_string(),
                    foreground: TermColor::Grey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
            return;
        }

        let total_width = self.hand.len() * (card_width() + 1);
        let start_column = ((screen_width - total_width as f64) / 2.0).max(0.0);
        let card_top_row = screen_height - card_height() as f64 - 3.0;

        let hint_text = "Left/Right: select    Enter: play card    E: end turn";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: screen_width / 2.0 - hint_text.len() as f64 / 2.0,
                row: card_top_row - 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: hint_text.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        for (hand_index, card) in self.hand.iter().enumerate() {
            let is_selected = hand_index == self.selected_card_index;
            let column_offset = start_column + hand_index as f64 * (card_width() + 1) as f64;
            let row_offset = if is_selected {
                card_top_row - 1.0
            } else {
                card_top_row
            };

            let border_color = card_border_color(card.card_type);
            let background = if is_selected {
                TermColor::Rgb {
                    r: 30,
                    g: 30,
                    b: 50,
                }
            } else {
                TermColor::Rgb {
                    r: 15,
                    g: 15,
                    b: 25,
                }
            };

            let can_play = card.energy_cost <= self.energy;
            let name_color = if can_play {
                TermColor::White
            } else {
                TermColor::DarkGrey
            };

            let width = card_width();
            let height = card_height();

            let top_border: String = format!("┌{}┐", "─".repeat(width - 2));
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: top_border,
                    foreground: border_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let padded_name = format!("{:^width$}", card.name, width = width - 2);
            let name_line = format!("│{}│", padded_name);
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 1.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: name_line,
                    foreground: name_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let separator: String = format!("├{}┤", "─".repeat(width - 2));
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 2.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: separator,
                    foreground: border_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let padded_desc = format!("{:^width$}", card.description, width = width - 2);
            let desc_line = format!("│{}│", padded_desc);
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 3.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: desc_line,
                    foreground: TermColor::Grey,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let empty_middle = format!("│{}│", " ".repeat(width - 2));
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 4.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: empty_middle,
                    foreground: border_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let energy_display = format!("{} energy", card.energy_cost);
            let padded_energy = format!("{:^width$}", energy_display, width = width - 2);
            let energy_line = format!("│{}│", padded_energy);
            let energy_color = if can_play {
                TermColor::Cyan
            } else {
                TermColor::DarkGrey
            };
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 5.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: energy_line,
                    foreground: energy_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let bottom_border: String = format!("└{}┘", "─".repeat(width - 2));
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + (height - 1) as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: bottom_border,
                    foreground: border_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            if is_selected {
                let indicator = "^";
                let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: column_offset + width as f64 / 2.0,
                        row: row_offset + height as f64,
                    },
                );
                world.set_label(
                    entity,
                    Label {
                        text: indicator.to_string(),
                        foreground: TermColor::Yellow,
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(11));
            }
        }
    }

    fn render_log(&mut self, world: &mut World, screen_width: f64, screen_height: f64) {
        let log_max_lines = 5;
        let log_start_row = screen_height - 2.0;
        let log_column = screen_width - 45.0;

        let visible_messages: Vec<&String> =
            self.combat_log.iter().rev().take(log_max_lines).collect();

        for (line_index, message) in visible_messages.iter().rev().enumerate() {
            let display_row = log_start_row - (log_max_lines - 1 - line_index) as f64;
            let truncated = if message.len() > 43 {
                format!("{}...", &message[..40])
            } else {
                message.to_string()
            };
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: log_column.max(0.0),
                    row: display_row,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: truncated,
                    foreground: TermColor::DarkGrey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }
    }
}

impl State for CombatState {
    fn title(&self) -> &str {
        "Deck Builder - Combat"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        self.start_combat();
        self.render_all(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        if self.transition != CombatTransition::None {
            return;
        }

        match key {
            KeyCode::Left => {
                if !self.hand.is_empty() && self.selected_card_index > 0 {
                    self.selected_card_index -= 1;
                    self.render_all(world);
                }
            }
            KeyCode::Right => {
                if !self.hand.is_empty() && self.selected_card_index + 1 < self.hand.len() {
                    self.selected_card_index += 1;
                    self.render_all(world);
                }
            }
            KeyCode::Enter => {
                self.play_selected_card(world);
                self.render_all(world);
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.end_turn();
                self.render_all(world);
            }
            KeyCode::Escape => {
                world.resources.should_exit = true;
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;
        self.particles.update(world, delta);
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        match self.transition {
            CombatTransition::Victory => {
                self.entities.despawn_all(world);
                self.particles.despawn_all(world);

                if self.is_boss {
                    return Some(Box::new(VictoryState {
                        entities: EntityGroup::new(),
                        restart: false,
                    }));
                }

                Some(Box::new(RewardState::new(
                    self.deck.clone(),
                    self.player_hp,
                    self.player_max_hp,
                    self.fight_number,
                )))
            }
            CombatTransition::Defeat => {
                self.entities.despawn_all(world);
                self.particles.despawn_all(world);
                Some(Box::new(DefeatState {
                    entities: EntityGroup::new(),
                    fight_number: self.fight_number,
                    is_boss: self.is_boss,
                    restart: false,
                }))
            }
            CombatTransition::None => None,
        }
    }
}

struct RewardState {
    deck: Vec<Card>,
    player_hp: i32,
    player_max_hp: i32,
    fight_number: usize,
    reward_options: Vec<Card>,
    selected_reward_index: usize,
    chosen: bool,
    skipped: bool,
    entities: EntityGroup,
}

impl RewardState {
    fn new(deck: Vec<Card>, player_hp: i32, player_max_hp: i32, fight_number: usize) -> Self {
        let pool = reward_card_pool();
        let mut rng = rand::rng();
        let mut chosen_types = Vec::new();
        while chosen_types.len() < 3 {
            let candidate = pool[rng.random_range(0..pool.len())];
            if !chosen_types.contains(&candidate) {
                chosen_types.push(candidate);
            }
        }
        let reward_options: Vec<Card> = chosen_types.into_iter().map(make_card).collect();

        Self {
            deck,
            player_hp,
            player_max_hp,
            fight_number,
            reward_options,
            selected_reward_index: 0,
            chosen: false,
            skipped: false,
            entities: EntityGroup::new(),
        }
    }

    fn render(&mut self, world: &mut World) {
        self.entities.despawn_all(world);

        let terminal = world.resources.terminal_size;
        let screen_width = terminal.columns as f64;
        let center_column = screen_width / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title = "VICTORY! Choose a card to add to your deck:";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - title.len() as f64 / 2.0,
                row: center_row - 8.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: title.to_string(),
                foreground: TermColor::Green,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let total_width = 3 * (card_width() + 2);
        let start_column = ((screen_width - total_width as f64) / 2.0).max(0.0);
        let card_top_row = center_row - 5.0;

        for (option_index, card) in self.reward_options.iter().enumerate() {
            let is_selected = option_index == self.selected_reward_index;
            let column_offset = start_column + option_index as f64 * (card_width() + 2) as f64;
            let row_offset = if is_selected {
                card_top_row - 1.0
            } else {
                card_top_row
            };

            let border_color = card_border_color(card.card_type);
            let background = if is_selected {
                TermColor::Rgb {
                    r: 30,
                    g: 30,
                    b: 50,
                }
            } else {
                TermColor::Rgb {
                    r: 15,
                    g: 15,
                    b: 25,
                }
            };
            let name_color = TermColor::White;

            let width = card_width();

            let top_border = format!("┌{}┐", "─".repeat(width - 2));
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: top_border,
                    foreground: border_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let padded_name = format!("{:^width$}", card.name, width = width - 2);
            let name_line = format!("│{}│", padded_name);
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 1.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: name_line,
                    foreground: name_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let separator = format!("├{}┤", "─".repeat(width - 2));
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 2.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: separator,
                    foreground: border_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let padded_desc = format!("{:^width$}", card.description, width = width - 2);
            let desc_line = format!("│{}│", padded_desc);
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 3.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: desc_line,
                    foreground: TermColor::Grey,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let empty_middle = format!("│{}│", " ".repeat(width - 2));
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 4.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: empty_middle,
                    foreground: border_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let energy_display = format!("{} energy", card.energy_cost);
            let padded_energy = format!("{:^width$}", energy_display, width = width - 2);
            let energy_line = format!("│{}│", padded_energy);
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 5.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: energy_line,
                    foreground: TermColor::Cyan,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            let bottom_border = format!("└{}┘", "─".repeat(width - 2));
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: column_offset,
                    row: row_offset + 6.0,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: bottom_border,
                    foreground: border_color,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(11));

            if is_selected {
                let indicator = "^";
                let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
                world.set_position(
                    entity,
                    Position {
                        column: column_offset + width as f64 / 2.0,
                        row: row_offset + 7.0,
                    },
                );
                world.set_label(
                    entity,
                    Label {
                        text: indicator.to_string(),
                        foreground: TermColor::Yellow,
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(11));
            }
        }

        let help_text = "Left/Right: select    Enter: add to deck    S: skip";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - help_text.len() as f64 / 2.0,
                row: card_top_row + card_height() as f64 + 2.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: help_text.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let deck_info = format!("Deck size: {} cards", self.deck.len());
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - deck_info.len() as f64 / 2.0,
                row: card_top_row + card_height() as f64 + 4.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: deck_info,
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));
    }
}

impl State for RewardState {
    fn title(&self) -> &str {
        "Deck Builder - Reward"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        self.render(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Left => {
                if self.selected_reward_index > 0 {
                    self.selected_reward_index -= 1;
                    self.render(world);
                }
            }
            KeyCode::Right => {
                if self.selected_reward_index + 1 < self.reward_options.len() {
                    self.selected_reward_index += 1;
                    self.render(world);
                }
            }
            KeyCode::Enter => {
                let chosen_card = self.reward_options[self.selected_reward_index].clone();
                self.deck.push(chosen_card);
                self.chosen = true;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.skipped = true;
            }
            KeyCode::Escape => {
                world.resources.should_exit = true;
            }
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.chosen || self.skipped {
            self.entities.despawn_all(world);

            let next_fight = self.fight_number + 1;
            if next_fight >= TOTAL_REGULAR_FIGHTS {
                return Some(Box::new(CombatState::new(
                    self.deck.clone(),
                    self.player_hp,
                    self.player_max_hp,
                    next_fight,
                    true,
                )));
            }

            return Some(Box::new(CombatState::new(
                self.deck.clone(),
                self.player_hp,
                self.player_max_hp,
                next_fight,
                false,
            )));
        }
        None
    }
}

struct VictoryState {
    entities: EntityGroup,
    restart: bool,
}

impl State for VictoryState {
    fn title(&self) -> &str {
        "Deck Builder - Victory!"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let lines: Vec<(&str, TermColor)> = vec![
            ("YOU WIN!", TermColor::Yellow),
            ("", TermColor::Black),
            ("You defeated The Guardian and", TermColor::White),
            ("conquered the spire!", TermColor::White),
            ("", TermColor::Black),
            ("Press R to play again", TermColor::Green),
            ("Press ESC to quit", TermColor::Grey),
        ];

        for (line_index, (text, color)) in lines.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - text.len() as f64 / 2.0,
                    row: center_row - 4.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: text.to_string(),
                    foreground: *color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Char('r') | KeyCode::Char('R') => self.restart = true,
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.restart {
            self.entities.despawn_all(world);
            return Some(Box::new(TitleScreenState {
                entities: EntityGroup::new(),
                start_game: false,
            }));
        }
        None
    }
}

struct DefeatState {
    entities: EntityGroup,
    fight_number: usize,
    is_boss: bool,
    restart: bool,
}

impl State for DefeatState {
    fn title(&self) -> &str {
        "Deck Builder - Defeat"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let defeat_detail = if self.is_boss {
            "Defeated by The Guardian".to_string()
        } else {
            format!("Fell in fight {}", self.fight_number + 1)
        };

        let lines: Vec<(String, TermColor)> = vec![
            ("DEFEAT".to_string(), TermColor::Red),
            (String::new(), TermColor::Black),
            (defeat_detail, TermColor::Yellow),
            (String::new(), TermColor::Black),
            ("Press R to try again".to_string(), TermColor::Green),
            ("Press ESC to quit".to_string(), TermColor::Grey),
        ];

        for (line_index, (text, color)) in lines.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - text.len() as f64 / 2.0,
                    row: center_row - 3.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: text.clone(),
                    foreground: *color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Char('r') | KeyCode::Char('R') => self.restart = true,
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.restart {
            self.entities.despawn_all(world);
            return Some(Box::new(TitleScreenState {
                entities: EntityGroup::new(),
                start_game: false,
            }));
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(TitleScreenState {
        entities: EntityGroup::new(),
        start_game: false,
    }))
}
