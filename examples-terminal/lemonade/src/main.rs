use nightshade::tui::prelude::*;
use rand::Rng;

const TOTAL_DAYS: u32 = 14;
const STARTING_MONEY: f64 = 20.0;
const CUP_COST: f64 = 0.50;
const LEMON_COST: f64 = 0.25;
const SUGAR_COST: f64 = 0.10;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Weather {
    Sunny,
    Cloudy,
    Rainy,
}

impl Weather {
    fn name(self) -> &'static str {
        match self {
            Self::Sunny => "Sunny",
            Self::Cloudy => "Cloudy",
            Self::Rainy => "Rainy",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Sunny => "(*)",
            Self::Cloudy => "(~)",
            Self::Rainy => "(/)",
        }
    }

    fn color(self) -> TermColor {
        match self {
            Self::Sunny => TermColor::Yellow,
            Self::Cloudy => TermColor::Grey,
            Self::Rainy => TermColor::Rgb {
                r: 100,
                g: 150,
                b: 255,
            },
        }
    }

    fn demand_multiplier(self) -> f64 {
        match self {
            Self::Sunny => 1.5,
            Self::Cloudy => 1.0,
            Self::Rainy => 0.4,
        }
    }

    fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..10) {
            0..=4 => Self::Sunny,
            5..=7 => Self::Cloudy,
            _ => Self::Rainy,
        }
    }
}

struct TitleScreenState {
    entities: EntityGroup,
    start_game: bool,
}

impl State for TitleScreenState {
    fn title(&self) -> &str {
        "Lemonade Stand - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let center_row = terminal.rows as f64 / 2.0;

        let title_lines = [
            "  _                                        _       ",
            " | |    ___ _ __ ___   ___  _ __   __ _  __| | ___  ",
            " | |   / _ \\ '_ ` _ \\ / _ \\| '_ \\ / _` |/ _` |/ _ \\ ",
            " | |__|  __/ | | | | | (_) | | | | (_| | (_| |  __/ ",
            " |_____\\___|_| |_| |_|\\___/|_| |_|\\__,_|\\__,_|\\___| ",
            "            ____  _                  _               ",
            "           / ___|| |_ __ _ _ __   __| |              ",
            "           \\___ \\| __/ _` | '_ \\ / _` |              ",
            "            ___) | || (_| | | | | (_| |              ",
            "           |____/ \\__\\__,_|_| |_|\\__,_|              ",
        ];

        for (line_index, line) in title_lines.iter().enumerate() {
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: center_column - line.len() as f64 / 2.0,
                    row: center_row - 8.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: line.to_string(),
                    foreground: TermColor::Yellow,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(10));
        }

        let subtitle = format!("Run your lemonade stand for {} days!", TOTAL_DAYS);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - subtitle.len() as f64 / 2.0,
                row: center_row + 4.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: subtitle,
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(10));

        let prompt = "Press ENTER to start";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - prompt.len() as f64 / 2.0,
                row: center_row + 6.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: prompt.to_string(),
                foreground: TermColor::Grey,
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
                row: center_row + 8.0,
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
            return Some(Box::new(DaySetupState::new()));
        }
        None
    }
}

struct DaySetupState {
    day: u32,
    money: f64,
    cups_owned: u32,
    lemons_owned: u32,
    sugar_owned: u32,
    price_cents: u32,
    cups_to_buy: u32,
    lemons_to_buy: u32,
    sugar_to_buy: u32,
    weather: Weather,
    selected_row: usize,
    entities: EntityGroup,
    proceed: bool,
    daily_profits: Vec<f64>,
    total_cups_sold: u32,
}

impl DaySetupState {
    fn new() -> Self {
        Self {
            day: 1,
            money: STARTING_MONEY,
            cups_owned: 0,
            lemons_owned: 0,
            sugar_owned: 0,
            price_cents: 25,
            cups_to_buy: 0,
            lemons_to_buy: 0,
            sugar_to_buy: 0,
            weather: Weather::random(),
            selected_row: 0,
            entities: EntityGroup::new(),
            proceed: false,
            daily_profits: Vec::new(),
            total_cups_sold: 0,
        }
    }

    fn from_previous(
        day: u32,
        money: f64,
        cups: u32,
        lemons: u32,
        sugar: u32,
        profits: Vec<f64>,
        total_sold: u32,
    ) -> Self {
        Self {
            day,
            money,
            cups_owned: cups,
            lemons_owned: lemons,
            sugar_owned: sugar,
            price_cents: 25,
            cups_to_buy: 0,
            lemons_to_buy: 0,
            sugar_to_buy: 0,
            weather: Weather::random(),
            selected_row: 0,
            entities: EntityGroup::new(),
            proceed: false,
            daily_profits: profits,
            total_cups_sold: total_sold,
        }
    }

    fn total_purchase_cost(&self) -> f64 {
        self.cups_to_buy as f64 * CUP_COST
            + self.lemons_to_buy as f64 * LEMON_COST
            + self.sugar_to_buy as f64 * SUGAR_COST
    }

    fn can_afford(&self) -> bool {
        self.total_purchase_cost() <= self.money + 0.001
    }

    fn max_servable(&self) -> u32 {
        let total_cups = self.cups_owned + self.cups_to_buy;
        let total_lemons = self.lemons_owned + self.lemons_to_buy;
        let total_sugar = self.sugar_owned + self.sugar_to_buy;
        total_cups.min(total_lemons).min(total_sugar)
    }

    fn render_all(&mut self, world: &mut World) {
        self.entities.despawn_all(world);

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let left_column = center_column - 30.0;

        let header = format!("=== Day {} of {} ===", self.day, TOTAL_DAYS);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - header.len() as f64 / 2.0,
                row: 1.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: header,
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));

        let weather_text = format!(
            "Weather Forecast: {} {}",
            self.weather.icon(),
            self.weather.name()
        );
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: left_column,
                row: 3.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: weather_text,
                foreground: self.weather.color(),
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));

        let money_text = format!("Money: ${:.2}", self.money);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: left_column,
                row: 5.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: money_text,
                foreground: TermColor::Green,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));

        let inventory_header = "--- Inventory ---";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: left_column,
                row: 7.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: inventory_header.to_string(),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));

        let inv_lines = [
            format!("  Cups:   {} in stock", self.cups_owned),
            format!("  Lemons: {} in stock", self.lemons_owned),
            format!("  Sugar:  {} in stock", self.sugar_owned),
        ];
        for (line_index, text) in inv_lines.iter().enumerate() {
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: left_column,
                    row: 8.0 + line_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: text.clone(),
                    foreground: TermColor::Grey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(5));
        }

        let purchase_header = "--- Purchase Supplies ---";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: left_column,
                row: 12.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: purchase_header.to_string(),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));

        let purchase_items = [
            format!(
                "Cups   (${:.2} ea):  {:>3}  [</>]",
                CUP_COST, self.cups_to_buy
            ),
            format!(
                "Lemons (${:.2} ea):  {:>3}  [</>]",
                LEMON_COST, self.lemons_to_buy
            ),
            format!(
                "Sugar  (${:.2} ea):  {:>3}  [</>]",
                SUGAR_COST, self.sugar_to_buy
            ),
            format!(
                "Price per cup:     ${:.2}  [</>]",
                self.price_cents as f64 / 100.0
            ),
        ];

        for (item_index, text) in purchase_items.iter().enumerate() {
            let is_selected = self.selected_row == item_index;
            let foreground = if is_selected {
                TermColor::Black
            } else {
                TermColor::White
            };
            let background = if is_selected {
                TermColor::Yellow
            } else {
                TermColor::Black
            };
            let prefix = if is_selected { "> " } else { "  " };
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: left_column,
                    row: 13.0 + item_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: format!("{}{}", prefix, text),
                    foreground,
                    background,
                },
            );
            world.set_z_index(entity, ZIndex(5));
        }

        let cost_text = format!("Purchase cost: ${:.2}", self.total_purchase_cost());
        let cost_color = if self.can_afford() {
            TermColor::Green
        } else {
            TermColor::Red
        };
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: left_column,
                row: 18.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: cost_text,
                foreground: cost_color,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));

        let servable_text = format!("Can serve: {} cups", self.max_servable());
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: left_column,
                row: 19.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: servable_text,
                foreground: TermColor::Cyan,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));

        let controls = "Up/Down: select | Left/Right: adjust | ENTER: start day";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - controls.len() as f64 / 2.0,
                row: 21.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: controls.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));
    }
}

impl State for DaySetupState {
    fn title(&self) -> &str {
        "Lemonade Stand - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;
        self.render_all(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Up => {
                if self.selected_row > 0 {
                    self.selected_row -= 1;
                }
                self.render_all(world);
            }
            KeyCode::Down => {
                if self.selected_row < 3 {
                    self.selected_row += 1;
                }
                self.render_all(world);
            }
            KeyCode::Left => {
                match self.selected_row {
                    0 => self.cups_to_buy = self.cups_to_buy.saturating_sub(1),
                    1 => self.lemons_to_buy = self.lemons_to_buy.saturating_sub(1),
                    2 => self.sugar_to_buy = self.sugar_to_buy.saturating_sub(1),
                    3 => self.price_cents = self.price_cents.saturating_sub(5).max(5),
                    _ => {}
                }
                self.render_all(world);
            }
            KeyCode::Right => {
                match self.selected_row {
                    0 => self.cups_to_buy += 1,
                    1 => self.lemons_to_buy += 1,
                    2 => self.sugar_to_buy += 1,
                    3 => self.price_cents = (self.price_cents + 5).min(200),
                    _ => {}
                }
                self.render_all(world);
            }
            KeyCode::Enter => {
                if self.can_afford() {
                    self.proceed = true;
                }
            }
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.proceed {
            let money_after = self.money - self.total_purchase_cost();
            let total_cups = self.cups_owned + self.cups_to_buy;
            let total_lemons = self.lemons_owned + self.lemons_to_buy;
            let total_sugar = self.sugar_owned + self.sugar_to_buy;
            let servable = total_cups.min(total_lemons).min(total_sugar);
            let price = self.price_cents as f64 / 100.0;

            let mut rng = rand::rng();
            let base_demand = rng.random_range(5..=20) as f64;
            let price_factor = if price <= 0.10 {
                2.0
            } else if price <= 0.25 {
                1.5
            } else if price <= 0.50 {
                1.0
            } else if price <= 1.00 {
                0.6
            } else {
                0.3
            };
            let demand =
                (base_demand * self.weather.demand_multiplier() * price_factor).round() as u32;
            let cups_sold = demand.min(servable);
            let revenue = cups_sold as f64 * price;
            let cost = self.total_purchase_cost();
            let profit = revenue - cost;

            let remaining_cups = total_cups - cups_sold;
            let remaining_lemons = total_lemons - cups_sold;
            let remaining_sugar = total_sugar - cups_sold;

            self.entities.despawn_all(world);

            return Some(Box::new(DayResultState {
                day: self.day,
                weather: self.weather,
                cups_sold,
                demand,
                servable,
                revenue,
                cost,
                profit,
                money: money_after + revenue,
                remaining_cups,
                remaining_lemons,
                remaining_sugar,
                entities: EntityGroup::new(),
                proceed: false,
                daily_profits: {
                    let mut profits = self.daily_profits.clone();
                    profits.push(profit);
                    profits
                },
                total_cups_sold: self.total_cups_sold + cups_sold,
            }));
        }
        None
    }
}

struct DayResultState {
    day: u32,
    weather: Weather,
    cups_sold: u32,
    demand: u32,
    servable: u32,
    revenue: f64,
    cost: f64,
    profit: f64,
    money: f64,
    remaining_cups: u32,
    remaining_lemons: u32,
    remaining_sugar: u32,
    entities: EntityGroup,
    proceed: bool,
    daily_profits: Vec<f64>,
    total_cups_sold: u32,
}

impl State for DayResultState {
    fn title(&self) -> &str {
        "Lemonade Stand - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let left_column = center_column - 25.0;

        let header = format!("=== Day {} Results ===", self.day);
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - header.len() as f64 / 2.0,
                row: 2.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: header,
                foreground: TermColor::Yellow,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));

        let weather_text = format!("Weather: {} {}", self.weather.icon(), self.weather.name());
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: left_column,
                row: 4.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: weather_text,
                foreground: self.weather.color(),
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));

        let lines: Vec<(String, TermColor)> = vec![
            (
                format!("Customer demand: {} cups", self.demand),
                TermColor::White,
            ),
            (
                format!("You could serve:  {} cups", self.servable),
                TermColor::White,
            ),
            (
                format!("Cups sold:        {} cups", self.cups_sold),
                TermColor::Cyan,
            ),
            (String::new(), TermColor::Black),
            (
                format!("Revenue:   +${:.2}", self.revenue),
                TermColor::Green,
            ),
            (format!("Expenses:  -${:.2}", self.cost), TermColor::Red),
            (
                format!("Profit:    ${:+.2}", self.profit),
                if self.profit >= 0.0 {
                    TermColor::Green
                } else {
                    TermColor::Red
                },
            ),
            (String::new(), TermColor::Black),
            (
                format!("Total money: ${:.2}", self.money),
                TermColor::Yellow,
            ),
        ];

        for (line_index, (text, color)) in lines.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: left_column,
                    row: 6.0 + line_index as f64,
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
            world.set_z_index(entity, ZIndex(5));
        }

        let prompt_text = if self.day < TOTAL_DAYS {
            "Press ENTER for next day"
        } else {
            "Press ENTER to see final results"
        };
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - prompt_text.len() as f64 / 2.0,
                row: 17.0,
            },
        );
        world.set_label(
            entity,
            Label {
                text: prompt_text.to_string(),
                foreground: TermColor::Grey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Enter => self.proceed = true,
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.proceed {
            self.entities.despawn_all(world);
            if self.day >= TOTAL_DAYS {
                return Some(Box::new(GameOverState {
                    final_money: self.money,
                    daily_profits: self.daily_profits.clone(),
                    total_cups_sold: self.total_cups_sold,
                    entities: EntityGroup::new(),
                    restart: false,
                }));
            }
            return Some(Box::new(DaySetupState::from_previous(
                self.day + 1,
                self.money,
                self.remaining_cups,
                self.remaining_lemons,
                self.remaining_sugar,
                self.daily_profits.clone(),
                self.total_cups_sold,
            )));
        }
        None
    }
}

struct GameOverState {
    final_money: f64,
    daily_profits: Vec<f64>,
    total_cups_sold: u32,
    entities: EntityGroup,
    restart: bool,
}

impl State for GameOverState {
    fn title(&self) -> &str {
        "Lemonade Stand - Final Results"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        let terminal = world.resources.terminal_size;
        let center_column = terminal.columns as f64 / 2.0;
        let left_column = center_column - 25.0;

        let title = "=== FINAL RESULTS ===";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - title.len() as f64 / 2.0,
                row: 2.0,
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
        world.set_z_index(entity, ZIndex(5));

        let total_profit: f64 = self.daily_profits.iter().sum();
        let best_day = self
            .daily_profits
            .iter()
            .enumerate()
            .max_by(|(_, profit_a), (_, profit_b)| {
                profit_a
                    .partial_cmp(profit_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, profit)| (index + 1, *profit));

        let rating = if self.final_money >= 100.0 {
            (
                "Lemonade Tycoon!",
                TermColor::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                },
            )
        } else if self.final_money >= 50.0 {
            ("Successful Entrepreneur", TermColor::Green)
        } else if self.final_money >= 20.0 {
            ("Broke Even", TermColor::Yellow)
        } else {
            ("Bankrupt!", TermColor::Red)
        };

        let lines: Vec<(String, TermColor)> = vec![
            (
                format!("Final Money:      ${:.2}", self.final_money),
                TermColor::Green,
            ),
            (
                format!("Starting Money:   ${:.2}", STARTING_MONEY),
                TermColor::Grey,
            ),
            (
                format!("Total Profit:     ${:+.2}", total_profit),
                if total_profit >= 0.0 {
                    TermColor::Green
                } else {
                    TermColor::Red
                },
            ),
            (
                format!("Cups Sold:        {}", self.total_cups_sold),
                TermColor::Cyan,
            ),
            (String::new(), TermColor::Black),
            (
                format!(
                    "Best Day: Day {} (${:+.2})",
                    best_day.map_or(0, |(day, _)| day),
                    best_day.map_or(0.0, |(_, profit)| profit)
                ),
                TermColor::Yellow,
            ),
            (String::new(), TermColor::Black),
            (format!("Rating: {}", rating.0), rating.1),
        ];

        for (line_index, (text, color)) in lines.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: left_column,
                    row: 5.0 + line_index as f64,
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
            world.set_z_index(entity, ZIndex(5));
        }

        let bar_row = 15.0;
        let bar_label = "Daily Profits:";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: left_column,
                row: bar_row,
            },
        );
        world.set_label(
            entity,
            Label {
                text: bar_label.to_string(),
                foreground: TermColor::White,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));

        let max_profit = self.daily_profits.iter().cloned().fold(0.1_f64, f64::max);
        for (day_index, profit) in self.daily_profits.iter().enumerate() {
            let bar_width = ((profit / max_profit) * 20.0).max(0.0) as usize;
            let bar: String = "#".repeat(bar_width);
            let bar_text = format!("D{:>2}: {:>6.2} {}", day_index + 1, profit, bar);
            let bar_color = if *profit >= 0.0 {
                TermColor::Green
            } else {
                TermColor::Red
            };
            let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
            world.set_position(
                entity,
                Position {
                    column: left_column,
                    row: bar_row + 1.0 + day_index as f64,
                },
            );
            world.set_label(
                entity,
                Label {
                    text: bar_text,
                    foreground: bar_color,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(5));
        }

        let prompt = "Press R to play again | ESC to quit";
        let entity = self.entities.spawn_one(world, POSITION | LABEL | Z_INDEX);
        world.set_position(
            entity,
            Position {
                column: center_column - prompt.len() as f64 / 2.0,
                row: bar_row + 2.0 + self.daily_profits.len() as f64,
            },
        );
        world.set_label(
            entity,
            Label {
                text: prompt.to_string(),
                foreground: TermColor::DarkGrey,
                background: TermColor::Black,
            },
        );
        world.set_z_index(entity, ZIndex(5));
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }
        match key {
            KeyCode::Char('r') => self.restart = true,
            KeyCode::Escape => world.resources.should_exit = true,
            _ => {}
        }
    }

    fn next_state(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        if self.restart {
            self.entities.despawn_all(world);
            return Some(Box::new(DaySetupState::new()));
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
