use nightshade::tui::prelude::*;
use std::collections::HashSet;

const ROOM_TOWER_BASE: usize = 0;
const ROOM_ENTRY_HALL: usize = 1;
const ROOM_KITCHEN: usize = 2;
const ROOM_SPIRAL_STAIRCASE: usize = 3;
const ROOM_LIBRARY: usize = 4;
const ROOM_STUDY: usize = 5;
const ROOM_UPPER_LANDING: usize = 6;
const ROOM_ARMORY: usize = 7;
const ROOM_OBSERVATORY: usize = 8;
const ROOM_TOWER_PEAK: usize = 9;

struct Room {
    name: &'static str,
    description: &'static str,
    exits: Vec<Exit>,
}

struct Exit {
    direction: &'static str,
    destination: usize,
    locked: bool,
    lock_flag: &'static str,
    lock_message: &'static str,
}

fn build_rooms() -> Vec<Room> {
    vec![
        Room {
            name: "Tower Base",
            description: "You stand at the base of a crumbling stone tower that stretches impossibly high into a bruised violet sky. The ground beneath your feet is rough-hewn granite, cracked and stained with age. Moss creeps along the walls in dark veins, and the air smells of damp earth and something older, something forgotten. A heavy wooden door leads north into the tower's interior. Near your feet, half-buried in the grit between the flagstones, you notice the dull glint of a rusty key.",
            exits: vec![Exit {
                direction: "north",
                destination: ROOM_ENTRY_HALL,
                locked: false,
                lock_flag: "",
                lock_message: "",
            }],
        },
        Room {
            name: "Entry Hall",
            description: "A grand hall stretches before you, its vaulted ceiling lost in shadow. Faded tapestries line the walls, their images reduced to threadbare ghosts of forgotten battles and long-dead kings. Dust motes drift through shafts of pale light that slip through cracks in the stonework. To the east, an archway opens into what appears to be a kitchen. A reinforced oak door to the north is bound with brass fittings and sealed with a heavy lock. A spiral staircase winds upward through a gap in the ceiling.",
            exits: vec![
                Exit {
                    direction: "south",
                    destination: ROOM_TOWER_BASE,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
                Exit {
                    direction: "east",
                    destination: ROOM_KITCHEN,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
                Exit {
                    direction: "north",
                    destination: ROOM_UPPER_LANDING,
                    locked: true,
                    lock_flag: "brass_door_unlocked",
                    lock_message: "The brass-bound door is locked tight. You need the right key.",
                },
                Exit {
                    direction: "up",
                    destination: ROOM_SPIRAL_STAIRCASE,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
            ],
        },
        Room {
            name: "Kitchen",
            description: "An old kitchen that once fed the tower's inhabitants. A blackened fireplace dominates the far wall, its iron grate choked with ancient ash. Copper pots hang from hooks overhead, tarnished green with age. A long wooden table sits in the center, scarred with knife marks. Against the wall, a row of drawers stands partially open, their contents long since scattered. One drawer, however, remains firmly shut.",
            exits: vec![Exit {
                direction: "west",
                destination: ROOM_ENTRY_HALL,
                locked: false,
                lock_flag: "",
                lock_message: "",
            }],
        },
        Room {
            name: "Spiral Staircase",
            description: "You stand on a narrow landing within a tightly wound spiral staircase. The stone steps are worn smooth by centuries of ascending and descending feet, their edges rounded into treacherous curves. The walls press close, and your shoulders nearly brush the cold stone on either side. A rusted iron railing offers uncertain support. The staircase continues both up and down, each direction swallowed by darkness.",
            exits: vec![
                Exit {
                    direction: "down",
                    destination: ROOM_ENTRY_HALL,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
                Exit {
                    direction: "up",
                    destination: ROOM_LIBRARY,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
            ],
        },
        Room {
            name: "Library",
            description: "Towering shelves of ancient books line every wall, reaching from floor to ceiling in columns of cracked leather and yellowed parchment. The smell of old paper is overwhelming, mingling with dust and the faint sweetness of decay. Many volumes have crumbled where they stand, their spines collapsed into illegible ruin. A heavy oak desk sits beneath a narrow window, its surface covered in scattered notes. One note in particular catches your eye, its ink still dark and legible.",
            exits: vec![
                Exit {
                    direction: "down",
                    destination: ROOM_SPIRAL_STAIRCASE,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
                Exit {
                    direction: "east",
                    destination: ROOM_STUDY,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
            ],
        },
        Room {
            name: "Study",
            description: "A scholar's private study, intimate and cluttered. Star charts paper the walls beside anatomical drawings and arcane diagrams. A leather chair sits before a writing desk piled with quills and inkwells. In the corner, a heavy iron chest squats on the floor, its lock mechanism intricate and well-oiled despite the dust that coats everything else. Whatever this chest protects, someone wanted it kept safe.",
            exits: vec![Exit {
                direction: "west",
                destination: ROOM_LIBRARY,
                locked: false,
                lock_flag: "",
                lock_message: "",
            }],
        },
        Room {
            name: "Upper Landing",
            description: "A wide landing opens at the top of the brass-bound stairway. A tall arched window dominates the eastern wall, its glass long shattered, letting in cold air that carries the scent of rain and distant lightning. Through the empty frame you can see the landscape far below, shrouded in mist. To the north stands an ornate door covered in celestial engravings that seem to shimmer faintly. To the east, a heavy curtain conceals another passage.",
            exits: vec![
                Exit {
                    direction: "south",
                    destination: ROOM_ENTRY_HALL,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
                Exit {
                    direction: "east",
                    destination: ROOM_ARMORY,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
                Exit {
                    direction: "north",
                    destination: ROOM_OBSERVATORY,
                    locked: true,
                    lock_flag: "observatory_unlocked",
                    lock_message: "The celestial door is sealed. A round depression in its center seems shaped to hold something spherical.",
                },
            ],
        },
        Room {
            name: "Armory",
            description: "Weapon racks line the walls of this long, narrow room, though most stand empty. The few remaining arms are pitted with rust, their edges dull and their hilts wrapped in rotting leather. A suit of plate armor stands in the corner like a hollow sentinel, one gauntlet raised as if in warning. On a hook beside the door frame, partially hidden behind a moth-eaten banner, hangs a dark iron key.",
            exits: vec![Exit {
                direction: "west",
                destination: ROOM_UPPER_LANDING,
                locked: false,
                lock_flag: "",
                lock_message: "",
            }],
        },
        Room {
            name: "Observatory",
            description: "You emerge into a domed chamber open to the sky. A massive brass telescope points upward through the shattered dome, its lenses clouded but intact. Star maps cover the curved walls, their constellations drawn in silver ink that still catches the light. Beneath the telescope, a pedestal holds a shallow bowl filled with dark water that reflects stars even in daylight. Resting on a velvet cloth beside the bowl, a golden amulet gleams with an inner warmth. The stairway continues upward to the tower's peak.",
            exits: vec![
                Exit {
                    direction: "south",
                    destination: ROOM_UPPER_LANDING,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
                Exit {
                    direction: "up",
                    destination: ROOM_TOWER_PEAK,
                    locked: false,
                    lock_flag: "",
                    lock_message: "",
                },
            ],
        },
        Room {
            name: "Tower Peak",
            description: "You stand at the very summit of the tower, exposed to the sky on all sides. The wind howls around you, tugging at your clothes and hair. The stone platform is small and circular, ringed by a low parapet carved with runes that pulse with faint blue light. At the center of the platform, a stone altar rises from the floor, its surface inscribed with a single circular depression that radiates warmth. The sky above churns with unnatural clouds, and you feel the weight of something ancient and powerful waiting just beyond the veil.",
            exits: vec![Exit {
                direction: "down",
                destination: ROOM_OBSERVATORY,
                locked: false,
                lock_flag: "",
                lock_message: "",
            }],
        },
    ]
}

fn build_room_items() -> Vec<Vec<String>> {
    let mut items = vec![Vec::new(); 10];
    items[ROOM_TOWER_BASE] = vec!["rusty key".to_string()];
    items[ROOM_KITCHEN] = vec!["brass key".to_string()];
    items[ROOM_LIBRARY] = vec!["cryptic note".to_string()];
    items[ROOM_ARMORY] = vec!["iron key".to_string()];
    items[ROOM_OBSERVATORY] = vec!["golden amulet".to_string()];
    items
}

fn item_description(item_name: &str) -> &'static str {
    match item_name {
        "rusty key" => {
            "A small iron key, pitted with rust and age. It looks like it might fit a simple lock."
        }
        "brass key" => {
            "A brass key with an ornate bow shaped like a lion's head. It feels solid and heavy in your hand."
        }
        "cryptic note" => {
            "A piece of yellowed parchment bearing a message in spidery handwriting: 'The orb reveals what the eye cannot see. Place it where the stars converge, and the way shall open.'"
        }
        "iron key" => {
            "A dark iron key, cold to the touch. Its teeth are complex and precisely cut, suggesting it opens something of importance."
        }
        "crystal orb" => {
            "A sphere of flawless crystal that fits perfectly in your palm. Inside, faint lights swirl like captured stars, casting prismatic reflections across nearby surfaces."
        }
        "golden amulet" => {
            "A heavy golden amulet on a chain of interlocking rings. At its center, a gem pulses with deep amber light, warm against your skin. Ancient runes circle the gem, too worn to read."
        }
        _ => "An unremarkable object.",
    }
}

fn examine_description(target: &str, current_room: usize) -> Option<&'static str> {
    match (target, current_room) {
        ("door", ROOM_TOWER_BASE) | ("wooden door", ROOM_TOWER_BASE) => Some(
            "A heavy wooden door reinforced with iron bands. It stands slightly ajar, revealing darkness beyond.",
        ),
        ("moss", ROOM_TOWER_BASE) => Some(
            "Dark green moss clings to the stonework in thick patches. It is damp to the touch and smells of old earth.",
        ),
        ("tapestries", ROOM_ENTRY_HALL) | ("tapestry", ROOM_ENTRY_HALL) => Some(
            "The tapestries are so faded that only the vaguest shapes remain: a crowned figure on horseback, a tower wreathed in flame, an army marching into shadow.",
        ),
        ("fireplace", ROOM_KITCHEN) => Some(
            "The fireplace is large enough to roast an ox. Its stones are blackened with centuries of soot. No warmth remains here.",
        ),
        ("drawer", ROOM_KITCHEN) | ("drawers", ROOM_KITCHEN) => Some(
            "Most drawers are empty or contain only mouse droppings and dust. One drawer, though, has a small brass key tucked in the back corner.",
        ),
        ("railing", ROOM_SPIRAL_STAIRCASE) => Some(
            "The iron railing is rough with rust. It wobbles alarmingly when you grip it, but holds.",
        ),
        ("books", ROOM_LIBRARY) | ("shelves", ROOM_LIBRARY) => Some(
            "The books cover every subject imaginable: alchemy, astronomy, history, philosophy, and stranger disciplines you cannot name. Many have fallen apart where they stand.",
        ),
        ("desk", ROOM_LIBRARY) => Some(
            "The oak desk is massive and scarred. Among the scattered papers, one note stands out with its dark, fresh-looking ink.",
        ),
        ("note", ROOM_LIBRARY) => Some(
            "The note reads: 'The orb reveals what the eye cannot see. Place it where the stars converge, and the way shall open.'",
        ),
        ("chest", ROOM_STUDY) => Some(
            "The iron chest is solidly built, its lock mechanism showing no signs of rust. Whatever magic preserves it must be powerful.",
        ),
        ("charts", ROOM_STUDY) | ("diagrams", ROOM_STUDY) | ("star charts", ROOM_STUDY) => Some(
            "The charts map constellations you do not recognize. Lines connect stars in patterns that seem to shift when you look away.",
        ),
        ("window", ROOM_UPPER_LANDING) => Some(
            "Through the shattered window frame, the world below is a patchwork of mist and shadow. You can see for miles, but nothing moves in the landscape.",
        ),
        ("door", ROOM_UPPER_LANDING) | ("celestial door", ROOM_UPPER_LANDING) => Some(
            "The ornate door is covered in engravings of stars, moons, and celestial bodies. At its center, a round depression waits to receive something spherical.",
        ),
        ("armor", ROOM_ARMORY) | ("suit", ROOM_ARMORY) => Some(
            "The plate armor stands frozen in a posture of warning. Through the visor slit, there is only darkness. You feel oddly watched.",
        ),
        ("weapons", ROOM_ARMORY) | ("racks", ROOM_ARMORY) => Some(
            "The weapon racks hold only a handful of rusted swords and a broken halberd. Nothing here is fit for use.",
        ),
        ("telescope", ROOM_OBSERVATORY) => Some(
            "The massive brass telescope is fixed in place, aimed at a patch of sky that should hold nothing remarkable, yet you feel drawn to look through it.",
        ),
        ("bowl", ROOM_OBSERVATORY) | ("water", ROOM_OBSERVATORY) => Some(
            "The dark water in the bowl is perfectly still. It reflects stars that are not in the sky above. You see constellations that match the charts from the study.",
        ),
        ("altar", ROOM_TOWER_PEAK) => Some(
            "The stone altar pulses with warmth. The circular depression at its center is lined with the same runes that ring the parapet. It awaits something.",
        ),
        ("runes", ROOM_TOWER_PEAK) | ("parapet", ROOM_TOWER_PEAK) => Some(
            "The runes pulse with a faint blue glow, growing brighter when you draw near. They are written in a language you cannot read, but their meaning is clear: this place is a threshold.",
        ),
        ("sky", ROOM_TOWER_PEAK) | ("clouds", ROOM_TOWER_PEAK) => Some(
            "The clouds above churn in slow, deliberate spirals. Lightning flickers within them, but no thunder follows. Something waits beyond.",
        ),
        _ => None,
    }
}

struct AdventureState {
    current_room: usize,
    inventory: Vec<String>,
    room_items: Vec<Vec<String>>,
    flags: HashSet<String>,
    input_buffer: String,
    message_log: Vec<(String, TermColor)>,
    display_entities: Vec<Entity>,
    cursor_blink_timer: f64,
    cursor_visible: bool,
    game_won: bool,
    rooms: Vec<Room>,
    needs_redraw: bool,
}

impl AdventureState {
    fn new() -> Self {
        Self {
            current_room: ROOM_TOWER_BASE,
            inventory: Vec::new(),
            room_items: build_room_items(),
            flags: HashSet::new(),
            input_buffer: String::new(),
            message_log: Vec::new(),
            display_entities: Vec::new(),
            cursor_blink_timer: 0.0,
            cursor_visible: true,
            game_won: false,
            rooms: build_rooms(),
            needs_redraw: true,
        }
    }

    fn add_message(&mut self, text: &str, color: TermColor) {
        self.message_log.push((text.to_string(), color));
        self.needs_redraw = true;
    }

    fn show_room(&mut self) {
        let room_name = self.rooms[self.current_room].name.to_string();
        let room_description = self.rooms[self.current_room].description.to_string();
        let exit_names: Vec<String> = self.rooms[self.current_room]
            .exits
            .iter()
            .map(|exit| exit.direction.to_string())
            .collect();
        let items: Vec<String> = self.room_items[self.current_room].clone();

        self.add_message(&format!("--- {} ---", room_name), TermColor::Cyan);
        self.add_message("", TermColor::White);
        self.add_message(&room_description, TermColor::White);
        self.add_message("", TermColor::White);

        self.add_message(
            &format!("Exits: {}", exit_names.join(", ")),
            TermColor::Green,
        );

        if !items.is_empty() {
            self.add_message(&format!("You see: {}", items.join(", ")), TermColor::Yellow);
        }

        self.add_message("", TermColor::White);
    }

    fn process_command(&mut self, command: &str) {
        let command = command.trim().to_lowercase();
        if command.is_empty() {
            return;
        }

        self.add_message(&format!("> {}", command), TermColor::DarkGrey);

        let parts: Vec<&str> = command.splitn(2, ' ').collect();
        let verb = parts[0];
        let noun = if parts.len() > 1 { parts[1].trim() } else { "" };

        match verb {
            "north" | "n" => self.try_move("north"),
            "south" | "s" => self.try_move("south"),
            "east" | "e" => self.try_move("east"),
            "west" | "w" => self.try_move("west"),
            "up" | "u" => self.try_move("up"),
            "down" | "d" => self.try_move("down"),
            "look" | "l" => self.show_room(),
            "take" | "get" | "pick" => self.take_item(noun),
            "drop" => self.drop_item(noun),
            "use" => self.use_item(noun),
            "inventory" | "inv" => self.show_inventory(),
            "examine" | "x" | "inspect" => self.examine(noun),
            "help" | "?" => self.show_help(),
            "quit" | "exit" => {
                self.add_message("Farewell, adventurer.", TermColor::Grey);
                self.flags.insert("quit_requested".to_string());
            }
            _ => {
                self.add_message(
                    "I don't understand that command. Type 'help' for a list of commands.",
                    TermColor::Red,
                );
            }
        }
    }

    fn try_move(&mut self, direction: &str) {
        let exit_data = self.rooms[self.current_room]
            .exits
            .iter()
            .find(|exit| exit.direction == direction)
            .map(|exit| {
                (
                    exit.destination,
                    exit.locked,
                    exit.lock_flag.to_string(),
                    exit.lock_message.to_string(),
                )
            });

        match exit_data {
            Some((destination, locked, lock_flag, lock_message)) => {
                if locked && !self.flags.contains(&lock_flag) {
                    self.add_message(&lock_message, TermColor::Red);
                } else {
                    self.current_room = destination;
                    self.add_message("", TermColor::White);
                    self.show_room();
                    self.check_victory();
                }
            }
            None => {
                self.add_message("You cannot go that way.", TermColor::Red);
            }
        }
    }

    fn take_item(&mut self, item_name: &str) {
        if item_name.is_empty() {
            self.add_message("Take what?", TermColor::Red);
            return;
        }

        let room_index = self.current_room;
        let found_index = self.room_items[room_index]
            .iter()
            .position(|item| item.to_lowercase().contains(item_name));

        match found_index {
            Some(index) => {
                let item = self.room_items[room_index].remove(index);
                let message = format!("You pick up the {}.", item);
                self.inventory.push(item);
                self.add_message(
                    &message,
                    TermColor::Rgb {
                        r: 100,
                        g: 255,
                        b: 100,
                    },
                );
            }
            None => {
                self.add_message("You don't see that here.", TermColor::Red);
            }
        }
    }

    fn drop_item(&mut self, item_name: &str) {
        if item_name.is_empty() {
            self.add_message("Drop what?", TermColor::Red);
            return;
        }

        let found_index = self
            .inventory
            .iter()
            .position(|item| item.to_lowercase().contains(item_name));

        match found_index {
            Some(index) => {
                let item = self.inventory.remove(index);
                let message = format!("You drop the {}.", item);
                self.room_items[self.current_room].push(item);
                self.add_message(
                    &message,
                    TermColor::Rgb {
                        r: 100,
                        g: 255,
                        b: 100,
                    },
                );
            }
            None => {
                self.add_message("You're not carrying that.", TermColor::Red);
            }
        }
    }

    fn use_item(&mut self, item_name: &str) {
        if item_name.is_empty() {
            self.add_message("Use what?", TermColor::Red);
            return;
        }

        let has_item = self
            .inventory
            .iter()
            .any(|item| item.to_lowercase().contains(item_name));

        if !has_item {
            let in_room = self.room_items[self.current_room]
                .iter()
                .any(|item| item.to_lowercase().contains(item_name));
            if in_room {
                self.add_message("You need to pick that up first.", TermColor::Red);
            } else {
                self.add_message("You don't have that.", TermColor::Red);
            }
            return;
        }

        match (item_name, self.current_room) {
            ("rusty key" | "rusty", ROOM_TOWER_BASE) => {
                self.add_message(
                    "There is nothing here to use the rusty key on.",
                    TermColor::Red,
                );
            }
            ("brass key" | "brass", ROOM_ENTRY_HALL) => {
                if self.flags.contains("brass_door_unlocked") {
                    self.add_message("The brass door is already unlocked.", TermColor::Grey);
                } else {
                    self.flags.insert("brass_door_unlocked".to_string());
                    self.add_message(
                        "You insert the brass key into the lock. It turns with a satisfying click, and the heavy oak door swings open, revealing a stairway leading up.",
                        TermColor::Rgb {
                            r: 100,
                            g: 255,
                            b: 100,
                        },
                    );
                    if let Some(index) = self.inventory.iter().position(|item| item == "brass key")
                    {
                        self.inventory.remove(index);
                        self.add_message(
                            "The brass key crumbles to dust in the lock.",
                            TermColor::Yellow,
                        );
                    }
                }
            }
            ("iron key" | "iron", ROOM_STUDY) => {
                if self.flags.contains("chest_opened") {
                    self.add_message("The chest is already open.", TermColor::Grey);
                } else {
                    self.flags.insert("chest_opened".to_string());
                    self.room_items[ROOM_STUDY].push("crystal orb".to_string());
                    self.add_message(
                        "You fit the iron key into the chest's lock. The mechanism turns smoothly, and the heavy lid swings open. Inside, resting on a cushion of faded velvet, a crystal orb pulses with captive starlight.",
                        TermColor::Rgb {
                            r: 100,
                            g: 255,
                            b: 100,
                        },
                    );
                    if let Some(index) = self.inventory.iter().position(|item| item == "iron key") {
                        self.inventory.remove(index);
                        self.add_message(
                            "The iron key dissolves into fine black dust.",
                            TermColor::Yellow,
                        );
                    }
                }
            }
            ("crystal orb" | "crystal" | "orb", ROOM_UPPER_LANDING) => {
                if self.flags.contains("observatory_unlocked") {
                    self.add_message("The celestial door is already open.", TermColor::Grey);
                } else {
                    self.flags.insert("observatory_unlocked".to_string());
                    self.add_message(
                        "You place the crystal orb into the depression in the celestial door. The orb flares with brilliant light, and the engravings on the door begin to glow. With a deep grinding of ancient stone, the door slides open, revealing a domed chamber beyond.",
                        TermColor::Rgb {
                            r: 100,
                            g: 255,
                            b: 100,
                        },
                    );
                    if let Some(index) =
                        self.inventory.iter().position(|item| item == "crystal orb")
                    {
                        self.inventory.remove(index);
                        self.add_message(
                            "The crystal orb remains embedded in the door, its light steady and warm.",
                            TermColor::Yellow,
                        );
                    }
                }
            }
            ("golden amulet" | "golden" | "amulet", ROOM_TOWER_PEAK) => {
                self.add_message("", TermColor::White);
                self.add_message(
                    "You lift the golden amulet and place it into the depression on the altar. The runes along the parapet blaze with blinding white light. The churning clouds above tear apart, revealing a sky of impossible depth, filled with stars that have not been seen from this world in a thousand years.",
                    TermColor::Rgb {
                        r: 255,
                        g: 215,
                        b: 0,
                    },
                );
                self.add_message("", TermColor::White);
                self.add_message(
                    "A beam of pure golden light erupts from the altar, piercing the heavens. You feel the tower tremble beneath you, not with destruction, but with awakening. The Forgotten Tower remembers. And now, so does the world.",
                    TermColor::Rgb {
                        r: 255,
                        g: 215,
                        b: 0,
                    },
                );
                self.add_message("", TermColor::White);
                self.add_message(
                    "*** VICTORY ***",
                    TermColor::Rgb {
                        r: 255,
                        g: 255,
                        b: 100,
                    },
                );
                self.add_message(
                    "You have completed The Forgotten Tower.",
                    TermColor::Rgb {
                        r: 255,
                        g: 255,
                        b: 100,
                    },
                );
                self.add_message("Press ESC to exit.", TermColor::Grey);
                self.game_won = true;
            }
            (_, _) => {
                self.add_message("You can't figure out how to use that here.", TermColor::Red);
            }
        }
    }

    fn show_inventory(&mut self) {
        if self.inventory.is_empty() {
            self.add_message("You are carrying nothing.", TermColor::Magenta);
        } else {
            self.add_message(
                &format!("Inventory: {}", self.inventory.join(", ")),
                TermColor::Magenta,
            );
        }
    }

    fn examine(&mut self, target: &str) {
        if target.is_empty() {
            self.add_message("Examine what?", TermColor::Red);
            return;
        }

        let inventory_match = self
            .inventory
            .iter()
            .find(|item| item.to_lowercase().contains(target))
            .cloned();

        if let Some(item) = inventory_match {
            self.add_message(item_description(&item), TermColor::White);
            return;
        }

        let room_item_match = self.room_items[self.current_room]
            .iter()
            .find(|item| item.to_lowercase().contains(target))
            .cloned();

        if let Some(item) = room_item_match {
            self.add_message(item_description(&item), TermColor::White);
            return;
        }

        if let Some(description) = examine_description(target, self.current_room) {
            self.add_message(description, TermColor::White);
            return;
        }

        self.add_message("You see nothing special about that.", TermColor::Grey);
    }

    fn show_help(&mut self) {
        self.add_message("--- Available Commands ---", TermColor::Cyan);
        self.add_message(
            "north/south/east/west/up/down (or n/s/e/w/u/d) - Move in a direction",
            TermColor::White,
        );
        self.add_message(
            "look (or l) - Look around the current room",
            TermColor::White,
        );
        self.add_message("take [item] - Pick up an item", TermColor::White);
        self.add_message(
            "drop [item] - Drop an item from inventory",
            TermColor::White,
        );
        self.add_message(
            "use [item] - Use an item in the current context",
            TermColor::White,
        );
        self.add_message(
            "examine [thing] (or x) - Examine something closely",
            TermColor::White,
        );
        self.add_message("inventory (or inv) - Show your inventory", TermColor::White);
        self.add_message("help - Show this help text", TermColor::White);
        self.add_message("quit - Leave the game", TermColor::White);
    }

    fn check_victory(&mut self) {
        if self.current_room == ROOM_TOWER_PEAK
            && !self.inventory.iter().any(|item| item == "golden amulet")
            && !self.game_won
        {
            self.add_message(
                "The altar pulses with warmth, but the barrier around it resists you. You sense that something is missing, something golden and ancient.",
                TermColor::Yellow,
            );
        }
    }

    fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
        if text.is_empty() {
            return vec![String::new()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                if word.len() > max_width {
                    let mut remaining = word;
                    while remaining.len() > max_width {
                        lines.push(remaining[..max_width].to_string());
                        remaining = &remaining[max_width..];
                    }
                    current_line = remaining.to_string();
                } else {
                    current_line = word.to_string();
                }
            } else if current_line.len() + 1 + word.len() > max_width {
                lines.push(current_line);
                current_line = word.to_string();
            } else {
                current_line.push(' ');
                current_line.push_str(word);
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    fn render_display(&mut self, world: &mut World) {
        world.despawn_entities(&self.display_entities);
        self.display_entities.clear();

        let terminal_columns = world.resources.terminal_size.columns as usize;
        let terminal_rows = world.resources.terminal_size.rows as usize;

        if terminal_columns < 10 || terminal_rows < 5 {
            return;
        }

        let padding = 2;
        let max_text_width = if terminal_columns > padding * 2 + 4 {
            terminal_columns - padding * 2
        } else {
            terminal_columns
        };

        let prompt_text = format!(
            "> {}{}",
            self.input_buffer,
            if self.cursor_visible { "_" } else { " " }
        );

        let separator_row = terminal_rows - 3;
        let prompt_row = terminal_rows - 2;
        let inventory_row = terminal_rows - 1;
        let available_log_rows = separator_row.saturating_sub(1);

        let mut wrapped_lines: Vec<(String, TermColor)> = Vec::new();

        for (text, color) in &self.message_log {
            if text.is_empty() {
                wrapped_lines.push((String::new(), *color));
            } else {
                let lines = Self::word_wrap(text, max_text_width);
                for line in lines {
                    wrapped_lines.push((line, *color));
                }
            }
        }

        let start_index = if wrapped_lines.len() > available_log_rows {
            wrapped_lines.len() - available_log_rows
        } else {
            0
        };

        let visible_lines = &wrapped_lines[start_index..];

        let first_row = if visible_lines.len() < available_log_rows {
            available_log_rows - visible_lines.len()
        } else {
            0
        };

        for (line_index, (text, color)) in visible_lines.iter().enumerate() {
            let row = first_row + line_index;
            for (char_index, character) in text.chars().enumerate() {
                let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
                world.set_position(
                    entity,
                    Position {
                        column: (padding + char_index) as f64,
                        row: row as f64,
                    },
                );
                world.set_sprite(
                    entity,
                    Sprite {
                        character,
                        foreground: *color,
                        background: TermColor::Black,
                    },
                );
                world.set_z_index(entity, ZIndex(1));
                self.display_entities.push(entity);
            }
        }

        for column_index in 0..terminal_columns {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: column_index as f64,
                    row: separator_row as f64,
                },
            );
            world.set_sprite(
                entity,
                Sprite {
                    character: if column_index == 0 || column_index == terminal_columns - 1 {
                        '+'
                    } else {
                        '-'
                    },
                    foreground: TermColor::DarkGrey,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(1));
            self.display_entities.push(entity);
        }

        for (char_index, character) in prompt_text.chars().enumerate() {
            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (padding + char_index) as f64,
                    row: prompt_row as f64,
                },
            );

            let foreground = if char_index < 2 {
                TermColor::DarkGreen
            } else {
                TermColor::White
            };

            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(1));
            self.display_entities.push(entity);
        }

        let inventory_text = if self.inventory.is_empty() {
            "Inventory: (empty)".to_string()
        } else {
            format!("Inventory: {}", self.inventory.join(", "))
        };

        let inventory_label_length = "Inventory:".len();

        for (char_index, character) in inventory_text.chars().enumerate() {
            if padding + char_index >= terminal_columns {
                break;
            }

            let entity = world.spawn_entities(POSITION | SPRITE | Z_INDEX, 1)[0];
            world.set_position(
                entity,
                Position {
                    column: (padding + char_index) as f64,
                    row: inventory_row as f64,
                },
            );

            let foreground = if char_index < inventory_label_length {
                TermColor::Magenta
            } else {
                TermColor::Yellow
            };

            world.set_sprite(
                entity,
                Sprite {
                    character,
                    foreground,
                    background: TermColor::Black,
                },
            );
            world.set_z_index(entity, ZIndex(1));
            self.display_entities.push(entity);
        }
    }
}

impl State for AdventureState {
    fn title(&self) -> &str {
        "The Forgotten Tower - Ember"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.timing.target_fps = 30;

        self.add_message(
            "T H E   F O R G O T T E N   T O W E R",
            TermColor::Rgb {
                r: 200,
                g: 170,
                b: 100,
            },
        );
        self.add_message("", TermColor::White);
        self.add_message(
            "You awaken on cold stone, your head throbbing and your memory a tattered veil. Above you, a crumbling tower claws at a sky the color of an old bruise. You do not remember how you came to be here. You do not remember your name. But deep in your bones, you feel the pull of something above, something waiting at the tower's peak.",
            TermColor::Rgb {
                r: 200,
                g: 200,
                b: 200,
            },
        );
        self.add_message("", TermColor::White);
        self.add_message("Type 'help' for a list of commands.", TermColor::Grey);
        self.add_message("", TermColor::White);

        self.show_room();
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }

        if self.flags.contains("quit_requested") || (self.game_won && key == KeyCode::Escape) {
            world.resources.should_exit = true;
            return;
        }

        if self.game_won {
            return;
        }

        match key {
            KeyCode::Escape => {
                world.resources.should_exit = true;
            }
            KeyCode::Enter => {
                let command = self.input_buffer.clone();
                self.input_buffer.clear();
                self.process_command(&command);
                if self.flags.contains("quit_requested") {
                    world.resources.should_exit = true;
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
                self.needs_redraw = true;
            }
            KeyCode::Char(character) => {
                self.input_buffer.push(character);
                self.needs_redraw = true;
            }
            _ => {}
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        let delta = world.resources.timing.delta_seconds;

        self.cursor_blink_timer += delta;
        if self.cursor_blink_timer >= 0.5 {
            self.cursor_blink_timer = 0.0;
            self.cursor_visible = !self.cursor_visible;
            self.needs_redraw = true;
        }

        if self.needs_redraw {
            self.render_display(world);
            self.needs_redraw = false;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Box::new(AdventureState::new()))
}
