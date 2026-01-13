use nightshade::prelude::*;
use stateless::statemachine;
use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(TextAdventureState::new())?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Item {
    Torch,
    Key,
    Map,
    Sword,
    Crown,
}

statemachine! {
    name: Room,
    derive_states: [Debug, Clone, PartialEq, Eq],
    derive_events: [Debug, Clone, PartialEq],
    transitions: {
        *TitleScreen + Start = Entrance,
        Entrance + GoNorth = GreatHall,
        Entrance + GoSouth = Forest,
        Entrance + GoEast = Courtyard,
        GreatHall + GoWest = Armory,
        GreatHall + GoEast = Library,
        GreatHall + GoNorth = ThroneRoom,
        GreatHall + GoSouth = Entrance,
        Armory + TakeWeapon = Victory,
        Armory + GoEast = GreatHall,
        Library + ReadBook = Defeat,
        Library + GoWest = GreatHall,
        ThroneRoom + GoNorth = SecretChamber,
        ThroneRoom + GoSouth = GreatHall,
        SecretChamber + TakeCrown = Victory,
        SecretChamber + GoSouth = ThroneRoom,
        Courtyard + GoDown = Dungeon,
        Courtyard + GoWest = Entrance,
        Dungeon + GoUp = Courtyard,
        Forest + GoNorth = Entrance,
        Forest + Wander = Forest,
        _ + Reset = TitleScreen,
    }
}

const MAX_TERMINAL_LINES: usize = 25;

struct TextAdventureState {
    room_state: RoomState,
    user_input: String,
    terminal_lines: Vec<String>,
    terminal_entities: Vec<Entity>,
    input_entity: Option<Entity>,
    scrollbar_entity: Option<Entity>,
    cursor_blink_counter: u32,
    scroll_offset: usize,
    inventory: HashSet<Item>,
    torch_lit: bool,
}

impl TextAdventureState {
    fn new() -> Self {
        Self {
            room_state: RoomState::default(),
            user_input: String::new(),
            terminal_lines: Vec::new(),
            terminal_entities: Vec::new(),
            input_entity: None,
            scrollbar_entity: None,
            cursor_blink_counter: 0,
            scroll_offset: 0,
            inventory: HashSet::new(),
            torch_lit: false,
        }
    }

    fn print_line(&mut self, line: &str) {
        self.terminal_lines.push(line.to_string());
        self.scroll_offset = 0;
    }

    fn print_room(&mut self) {
        self.print_line("");
        match self.room_state {
            RoomState::TitleScreen => {
                self.print_line("=== CASTLE OF SHADOWS ===");
                self.print_line("");
                self.print_line("A mysterious castle looms before you,");
                self.print_line("shrouded in eternal twilight.");
                self.print_line("Your quest: Claim the legendary Crown of Kings.");
                self.print_line("");
                self.print_line("Commands: start, begin, play");
            }
            RoomState::Entrance => {
                self.print_line("--- Castle Entrance ---");
                self.print_line("");
                self.print_line("You stand before massive oak doors.");
                self.print_line("To the north, the Great Hall beckons.");
                self.print_line("A dark forest lies to the south.");
                self.print_line("Overgrown courtyard extends east.");
                self.print_line("");
                if self.inventory.contains(&Item::Map) {
                    self.print_line("(Your map shows all locations)");
                    self.print_line("");
                }
                self.print_line("Commands: n/north/hall | s/south/forest | e/east/courtyard");
                self.print_line("          i/inv/inventory | l/look");
            }
            RoomState::GreatHall => {
                self.print_line("--- Great Hall ---");
                self.print_line("");
                self.print_line("A vast chamber stretches before you.");
                self.print_line("Tapestries depicting ancient battles");
                self.print_line("hang from towering stone walls.");
                self.print_line("Corridors branch west, east, and north.");
                self.print_line("");
                self.print_line("Commands: w/west/armory | e/east/library | n/north/throne");
                self.print_line("          s/south/entrance | i/inv | l/look");
            }
            RoomState::Armory => {
                self.print_line("--- Armory ---");
                self.print_line("");
                self.print_line("Weapons of ages past line the walls.");
                self.print_line("Most are rusted and worthless, but");
                if !self.inventory.contains(&Item::Sword) {
                    self.print_line("a magnificent sword gleams with power.");
                    self.print_line("");
                    self.print_line("Commands: take sword | e/east/hall | i/inv | l/look");
                } else {
                    self.print_line("empty racks mark where the sword once lay.");
                    self.print_line("");
                    self.print_line("Commands: e/east/hall | i/inv | l/look");
                }
            }
            RoomState::Library => {
                self.print_line("--- Library ---");
                self.print_line("");
                self.print_line("Towering shelves packed with ancient tomes");
                self.print_line("cast long shadows in the dim light.");
                self.print_line("One book lies open, radiating dark energy.");
                self.print_line("");
                self.print_line("Commands: read book | w/west/hall | i/inv | l/look");
            }
            RoomState::ThroneRoom => {
                self.print_line("--- Throne Room ---");
                self.print_line("");
                self.print_line("A magnificent throne of obsidian dominates");
                self.print_line("this chamber. Behind it, ornate stonework");
                if self.inventory.contains(&Item::Key) {
                    self.print_line("conceals a keyhole you can now access.");
                    self.print_line("");
                    self.print_line("Commands: n/north/unlock | s/south/hall | i/inv | l/look");
                } else {
                    self.print_line("hides secrets you cannot yet reach.");
                    self.print_line("");
                    self.print_line("Commands: s/south/hall | i/inv | l/look");
                }
            }
            RoomState::SecretChamber => {
                self.print_line("--- Secret Chamber ---");
                self.print_line("");
                self.print_line("You stand in a hidden vault, untouched");
                self.print_line("for centuries. Treasures beyond measure");
                if !self.inventory.contains(&Item::Crown) {
                    self.print_line("surround you. The Crown of Kings rests");
                    self.print_line("upon a pedestal, bathed in golden light.");
                    self.print_line("");
                    self.print_line("Commands: take crown | s/south/throne | i/inv | l/look");
                } else {
                    self.print_line("lie scattered, but the crown is yours.");
                    self.print_line("");
                    self.print_line("Commands: s/south/throne | i/inv | l/look");
                }
            }
            RoomState::Courtyard => {
                self.print_line("--- Overgrown Courtyard ---");
                self.print_line("");
                self.print_line("Nature has reclaimed this once-grand space.");
                self.print_line("Vines creep across cracked stone. A stone");
                if !self.inventory.contains(&Item::Map) {
                    self.print_line("staircase descends into darkness. A faded");
                    self.print_line("map lies pinned under a fallen statue.");
                    self.print_line("");
                    self.print_line("Commands: take map | d/down/dungeon | w/west/entrance");
                } else {
                    self.print_line("staircase descends into darkness.");
                    self.print_line("");
                    self.print_line("Commands: d/down/dungeon | w/west/entrance");
                }
                self.print_line("          i/inv | l/look");
            }
            RoomState::Dungeon => {
                self.print_line("--- Dank Dungeon ---");
                self.print_line("");
                if self.torch_lit {
                    self.print_line("Your torch illuminates the chamber.");
                    self.print_line("Chains hang from walls. In the flickering");
                    if !self.inventory.contains(&Item::Key) {
                        self.print_line("light, you spot a golden key on the floor.");
                        self.print_line("");
                        self.print_line("Commands: take key | u/up/courtyard | i/inv | l/look");
                    } else {
                        self.print_line("light, the empty chamber reveals nothing more.");
                        self.print_line("");
                        self.print_line("Commands: u/up/courtyard | i/inv | l/look");
                    }
                } else {
                    self.print_line("Absolute darkness surrounds you.");
                    self.print_line("You cannot see anything.");
                    if !self.inventory.contains(&Item::Torch) {
                        self.print_line("Your hands brush against something wooden...");
                        self.print_line("");
                        self.print_line("Commands: take torch | u/up/courtyard | i/inv | l/look");
                    } else {
                        self.print_line("");
                        self.print_line("Commands: light torch | u/up/courtyard | i/inv | l/look");
                    }
                }
            }
            RoomState::Forest => {
                self.print_line("--- Dark Forest ---");
                self.print_line("");
                self.print_line("Ancient trees tower overhead, their gnarled");
                self.print_line("branches blocking out the sky. Strange sounds");
                self.print_line("echo through the undergrowth. The castle");
                self.print_line("entrance lies to the north.");
                self.print_line("");
                self.print_line("Commands: n/north/entrance | wander/explore | i/inv | l/look");
            }
            RoomState::Victory => {
                self.print_line("*** VICTORY ***");
                self.print_line("");
                if self.inventory.contains(&Item::Crown) {
                    self.print_line("The Crown of Kings rests upon your head!");
                    self.print_line("Ancient power surges through you.");
                    self.print_line("You are the rightful ruler of this realm!");
                } else {
                    self.print_line("You grasp the legendary sword!");
                    self.print_line("Power courses through your veins.");
                    self.print_line("You are victorious!");
                }
                self.print_line("");
                self.print_line("YOU HAVE WON!");
                self.print_line("");
                self.print_line("Commands: restart/again");
            }
            RoomState::Defeat => {
                self.print_line("*** GAME OVER ***");
                self.print_line("");
                self.print_line("You read from the cursed tome!");
                self.print_line("Eldritch whispers fill your mind.");
                self.print_line("Darkness consumes your very soul.");
                self.print_line("");
                self.print_line("YOU HAVE DIED");
                self.print_line("");
                self.print_line("Commands: restart/again");
            }
        }
    }

    fn parse_command(&mut self, input: &str) -> Option<RoomEvent> {
        let input_lower = input.trim().to_lowercase();

        if input_lower.is_empty() {
            return None;
        }

        if input_lower == "i" || input_lower == "inv" || input_lower == "inventory" {
            self.show_inventory();
            return None;
        }

        if input_lower == "l" || input_lower == "look" {
            return None;
        }

        match self.room_state {
            RoomState::TitleScreen => {
                if input_lower.contains("start")
                    || input_lower.contains("begin")
                    || input_lower.contains("play")
                {
                    Some(RoomEvent::Start)
                } else {
                    None
                }
            }
            RoomState::Entrance => {
                if input_lower == "n"
                    || input_lower.contains("north")
                    || input_lower.contains("hall")
                {
                    Some(RoomEvent::GoNorth)
                } else if input_lower == "s"
                    || input_lower.contains("south")
                    || input_lower.contains("forest")
                {
                    Some(RoomEvent::GoSouth)
                } else if input_lower == "e"
                    || input_lower.contains("east")
                    || input_lower.contains("courtyard")
                {
                    Some(RoomEvent::GoEast)
                } else {
                    None
                }
            }
            RoomState::GreatHall => {
                if input_lower == "w"
                    || input_lower.contains("west")
                    || input_lower.contains("armory")
                {
                    Some(RoomEvent::GoWest)
                } else if input_lower == "e"
                    || input_lower.contains("east")
                    || input_lower.contains("library")
                {
                    Some(RoomEvent::GoEast)
                } else if input_lower == "n"
                    || input_lower.contains("north")
                    || input_lower.contains("throne")
                {
                    Some(RoomEvent::GoNorth)
                } else if input_lower == "s"
                    || input_lower.contains("south")
                    || input_lower.contains("entrance")
                {
                    Some(RoomEvent::GoSouth)
                } else {
                    None
                }
            }
            RoomState::Armory => {
                if input_lower.contains("take")
                    || input_lower.contains("grab")
                    || input_lower.contains("get")
                {
                    if input_lower.contains("sword") || input_lower.contains("weapon") {
                        if !self.inventory.contains(&Item::Sword) {
                            self.inventory.insert(Item::Sword);
                            self.print_line("");
                            self.print_line("You take the magnificent sword.");
                            self.print_line("It feels surprisingly light in your hands.");
                        } else {
                            self.print_line("");
                            self.print_line("You already have the sword.");
                        }
                        return None;
                    }
                    Some(RoomEvent::TakeWeapon)
                } else if input_lower == "e"
                    || input_lower.contains("east")
                    || input_lower.contains("hall")
                {
                    Some(RoomEvent::GoEast)
                } else {
                    None
                }
            }
            RoomState::Library => {
                if input_lower.contains("read")
                    || input_lower.contains("tome")
                    || input_lower.contains("book")
                    || input_lower.contains("open")
                {
                    Some(RoomEvent::ReadBook)
                } else if input_lower == "w"
                    || input_lower.contains("west")
                    || input_lower.contains("hall")
                {
                    Some(RoomEvent::GoWest)
                } else {
                    None
                }
            }
            RoomState::ThroneRoom => {
                if input_lower == "n"
                    || input_lower.contains("north")
                    || input_lower.contains("unlock")
                    || input_lower.contains("secret")
                {
                    if self.inventory.contains(&Item::Key) {
                        Some(RoomEvent::GoNorth)
                    } else {
                        self.print_line("");
                        self.print_line("The secret passage is locked.");
                        self.print_line("You need a key to unlock it.");
                        None
                    }
                } else if input_lower == "s"
                    || input_lower.contains("south")
                    || input_lower.contains("hall")
                {
                    Some(RoomEvent::GoSouth)
                } else {
                    None
                }
            }
            RoomState::SecretChamber => {
                if (input_lower.contains("take")
                    || input_lower.contains("grab")
                    || input_lower.contains("get"))
                    && input_lower.contains("crown")
                {
                    if !self.inventory.contains(&Item::Crown) {
                        self.inventory.insert(Item::Crown);
                        return Some(RoomEvent::TakeCrown);
                    } else {
                        self.print_line("");
                        self.print_line("You already have the crown.");
                        return None;
                    }
                }
                if input_lower == "s"
                    || input_lower.contains("south")
                    || input_lower.contains("throne")
                {
                    Some(RoomEvent::GoSouth)
                } else {
                    None
                }
            }
            RoomState::Courtyard => {
                if (input_lower.contains("take")
                    || input_lower.contains("grab")
                    || input_lower.contains("get"))
                    && input_lower.contains("map")
                {
                    if !self.inventory.contains(&Item::Map) {
                        self.inventory.insert(Item::Map);
                        self.print_line("");
                        self.print_line("You take the old map.");
                        self.print_line("It shows the layout of the castle.");
                    } else {
                        self.print_line("");
                        self.print_line("You already have the map.");
                    }
                    return None;
                }
                if input_lower == "d"
                    || input_lower.contains("down")
                    || input_lower.contains("dungeon")
                {
                    Some(RoomEvent::GoDown)
                } else if input_lower == "w"
                    || input_lower.contains("west")
                    || input_lower.contains("entrance")
                {
                    Some(RoomEvent::GoWest)
                } else {
                    None
                }
            }
            RoomState::Dungeon => {
                if input_lower.contains("light") && input_lower.contains("torch") {
                    if self.inventory.contains(&Item::Torch) {
                        if !self.torch_lit {
                            self.torch_lit = true;
                            self.print_line("");
                            self.print_line("You light the torch.");
                            self.print_line("Warm light floods the chamber.");
                            self.print_room();
                        } else {
                            self.print_line("");
                            self.print_line("The torch is already lit.");
                        }
                    } else {
                        self.print_line("");
                        self.print_line("You don't have a torch to light.");
                    }
                    return None;
                }
                if input_lower.contains("take")
                    || input_lower.contains("grab")
                    || input_lower.contains("get")
                {
                    if input_lower.contains("torch") {
                        if !self.inventory.contains(&Item::Torch) {
                            self.inventory.insert(Item::Torch);
                            self.print_line("");
                            self.print_line("You pick up the torch.");
                            self.print_line("You could light it...");
                        } else {
                            self.print_line("");
                            self.print_line("You already have the torch.");
                        }
                        return None;
                    } else if input_lower.contains("key") {
                        if self.torch_lit {
                            if !self.inventory.contains(&Item::Key) {
                                self.inventory.insert(Item::Key);
                                self.print_line("");
                                self.print_line("You take the golden key.");
                                self.print_line("It glimmers with ancient magic.");
                            } else {
                                self.print_line("");
                                self.print_line("You already have the key.");
                            }
                        } else {
                            self.print_line("");
                            self.print_line("It's too dark to find anything.");
                        }
                        return None;
                    }
                }
                if input_lower == "u"
                    || input_lower.contains("up")
                    || input_lower.contains("courtyard")
                {
                    Some(RoomEvent::GoUp)
                } else {
                    None
                }
            }
            RoomState::Forest => {
                if input_lower == "n"
                    || input_lower.contains("north")
                    || input_lower.contains("entrance")
                    || input_lower.contains("castle")
                {
                    Some(RoomEvent::GoNorth)
                } else if input_lower.contains("wander")
                    || input_lower.contains("walk")
                    || input_lower.contains("explore")
                    || input_lower.contains("deeper")
                {
                    Some(RoomEvent::Wander)
                } else {
                    None
                }
            }
            RoomState::Victory | RoomState::Defeat => {
                if input_lower.contains("reset")
                    || input_lower.contains("restart")
                    || input_lower.contains("again")
                    || input_lower.contains("play")
                {
                    Some(RoomEvent::Reset)
                } else {
                    None
                }
            }
        }
    }

    fn show_inventory(&mut self) {
        self.print_line("");
        if self.inventory.is_empty() {
            self.print_line("Your inventory is empty.");
        } else {
            self.print_line("You are carrying:");
            let mut items_list = Vec::new();
            for item in &self.inventory {
                let item_name = match item {
                    Item::Torch => {
                        if self.torch_lit {
                            "Torch (lit)"
                        } else {
                            "Torch (unlit)"
                        }
                    }
                    Item::Key => "Golden Key",
                    Item::Map => "Old Map",
                    Item::Sword => "Magnificent Sword",
                    Item::Crown => "Crown of Kings",
                };
                items_list.push(format!("  - {}", item_name));
            }
            for item_line in items_list {
                self.print_line(&item_line);
            }
        }
    }

    fn process_command(&mut self, _world: &mut World, command: &str) {
        if command.is_empty() {
            return;
        }

        self.print_line(&format!("> {}", command));

        let input_lower = command.trim().to_lowercase();
        if input_lower == "l" || input_lower == "look" {
            self.print_room();
            return;
        }

        if let Some(event) = self.parse_command(command) {
            if let Some(new_state) = self.room_state.process_event(event) {
                self.room_state = new_state;
                self.print_room();
            } else {
                self.print_line("You can't do that right now.");
            }
        } else {
            self.print_line("I don't understand that command.");
        }
    }

    fn update_terminal_display(&mut self, world: &mut World) {
        self.cursor_blink_counter += 1;

        let total_lines = self.terminal_lines.len();
        let visible_lines = MAX_TERMINAL_LINES;

        let end_index = total_lines.saturating_sub(self.scroll_offset);
        let start_index = end_index.saturating_sub(visible_lines);

        let visible_text: Vec<&str> = self.terminal_lines[start_index..end_index]
            .iter()
            .map(|s| s.as_str())
            .collect();
        let terminal_text = visible_text.join("\n");

        for entity in &self.terminal_entities {
            if let Some(hud_text) = world.get_hud_text(*entity) {
                let text_index = hud_text.text_index;
                world
                    .resources
                    .text_cache
                    .set_text(text_index, &terminal_text);
                if let Some(hud_text) = world.get_hud_text_mut(*entity) {
                    hud_text.dirty = true;
                }
            }
        }

        if let Some(entity) = self.scrollbar_entity
            && let Some(hud_text) = world.get_hud_text(entity)
        {
            let text_index = hud_text.text_index;

            let scrollbar_text = if total_lines > visible_lines {
                let max_scroll = total_lines - visible_lines;
                let scroll_percent = if max_scroll > 0 {
                    (max_scroll - self.scroll_offset) as f32 / max_scroll as f32
                } else {
                    1.0
                };

                let bar_height = 20;
                let bar_pos = (scroll_percent * (bar_height - 1) as f32) as usize;

                let mut bar_chars = vec!["|"; bar_height];
                bar_chars[bar_pos] = "#";

                format!(
                    "{}\n[{}-{}/{}]",
                    bar_chars.join("\n"),
                    start_index + 1,
                    end_index,
                    total_lines
                )
            } else {
                String::new()
            };

            world
                .resources
                .text_cache
                .set_text(text_index, &scrollbar_text);
            if let Some(hud_text) = world.get_hud_text_mut(entity) {
                hud_text.dirty = true;
            }
        }

        if let Some(entity) = self.input_entity
            && let Some(hud_text) = world.get_hud_text(entity)
        {
            let text_index = hud_text.text_index;
            let cursor = if (self.cursor_blink_counter / 30).is_multiple_of(2) {
                "_"
            } else {
                " "
            };
            let input_text = format!("> {}{}", self.user_input, cursor);
            world.resources.text_cache.set_text(text_index, &input_text);
            if let Some(hud_text) = world.get_hud_text_mut(entity) {
                hud_text.dirty = true;
            }
        }
    }
}

impl State for TextAdventureState {
    fn title(&self) -> &str {
        "Text Adventure - Castle of Shadows"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.show_grid = false;
        world.resources.graphics.atmosphere = Atmosphere::None;
        world.resources.graphics.clear_color = [0.0, 0.0, 0.0, 1.0];

        let camera_position = Vec3::new(0.0, 0.0, 5.0);
        let main_camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(main_camera);

        let terminal_props = TextProperties {
            font_size: 16.0,
            color: Vec4::new(0.0, 1.0, 0.0, 1.0),
            alignment: TextAlignment::Left,
            line_height: 1.2,
            ..Default::default()
        };

        let terminal_entity = spawn_hud_text_with_properties(
            world,
            "",
            HudAnchor::TopLeft,
            Vec2::new(20.0, -20.0),
            terminal_props,
        );
        self.terminal_entities.push(terminal_entity);

        let prompt_props = TextProperties {
            font_size: 16.0,
            color: Vec4::new(0.0, 1.0, 0.0, 1.0),
            alignment: TextAlignment::Left,
            ..Default::default()
        };

        self.input_entity = Some(spawn_hud_text_with_properties(
            world,
            "> _",
            HudAnchor::BottomLeft,
            Vec2::new(20.0, -30.0),
            prompt_props.clone(),
        ));

        let scrollbar_props = TextProperties {
            font_size: 12.0,
            color: Vec4::new(0.5, 0.5, 0.5, 1.0),
            alignment: TextAlignment::Right,
            line_height: 1.0,
            ..Default::default()
        };

        self.scrollbar_entity = Some(spawn_hud_text_with_properties(
            world,
            "",
            HudAnchor::TopRight,
            Vec2::new(-20.0, -20.0),
            scrollbar_props,
        ));

        self.print_room();
        self.update_terminal_display(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        self.update_terminal_display(world);
        sync_text_meshes_system(world);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key_code: KeyCode, key_state: KeyState) {
        if !matches!(key_state, KeyState::Pressed) {
            return;
        }

        match key_code {
            KeyCode::PageUp | KeyCode::ArrowUp => {
                let total_lines = self.terminal_lines.len();
                let visible_lines = MAX_TERMINAL_LINES;
                if total_lines > visible_lines {
                    let max_scroll = total_lines - visible_lines;
                    self.scroll_offset = (self.scroll_offset + 1).min(max_scroll);
                }
            }
            KeyCode::PageDown | KeyCode::ArrowDown => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::Home => {
                let total_lines = self.terminal_lines.len();
                let visible_lines = MAX_TERMINAL_LINES;
                if total_lines > visible_lines {
                    self.scroll_offset = total_lines - visible_lines;
                }
            }
            KeyCode::End => {
                self.scroll_offset = 0;
            }
            KeyCode::Enter => {
                let command = self.user_input.clone();
                self.user_input.clear();
                self.process_command(world, &command);
            }
            KeyCode::Backspace => {
                self.user_input.pop();
            }
            KeyCode::Escape => {
                world.resources.window.should_exit = true;
            }
            KeyCode::Space => {
                self.user_input.push(' ');
            }
            KeyCode::KeyA => self.user_input.push('a'),
            KeyCode::KeyB => self.user_input.push('b'),
            KeyCode::KeyC => self.user_input.push('c'),
            KeyCode::KeyD => self.user_input.push('d'),
            KeyCode::KeyE => self.user_input.push('e'),
            KeyCode::KeyF => self.user_input.push('f'),
            KeyCode::KeyG => self.user_input.push('g'),
            KeyCode::KeyH => self.user_input.push('h'),
            KeyCode::KeyI => self.user_input.push('i'),
            KeyCode::KeyJ => self.user_input.push('j'),
            KeyCode::KeyK => self.user_input.push('k'),
            KeyCode::KeyL => self.user_input.push('l'),
            KeyCode::KeyM => self.user_input.push('m'),
            KeyCode::KeyN => self.user_input.push('n'),
            KeyCode::KeyO => self.user_input.push('o'),
            KeyCode::KeyP => self.user_input.push('p'),
            KeyCode::KeyQ => self.user_input.push('q'),
            KeyCode::KeyR => self.user_input.push('r'),
            KeyCode::KeyS => self.user_input.push('s'),
            KeyCode::KeyT => self.user_input.push('t'),
            KeyCode::KeyU => self.user_input.push('u'),
            KeyCode::KeyV => self.user_input.push('v'),
            KeyCode::KeyW => self.user_input.push('w'),
            KeyCode::KeyX => self.user_input.push('x'),
            KeyCode::KeyY => self.user_input.push('y'),
            KeyCode::KeyZ => self.user_input.push('z'),
            _ => {}
        }
    }
}
