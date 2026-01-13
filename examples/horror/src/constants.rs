pub const FLOOR_TEXTURE: &[u8] = include_bytes!("../../../assets/textures/horror/floor.png");
pub const WALL_TEXTURE: &[u8] = include_bytes!("../../../assets/textures/horror/wall.png");
pub const CEILING_TEXTURE: &[u8] = include_bytes!("../../../assets/textures/horror/ceiling.png");
pub const DOOR_TEXTURE: &[u8] = include_bytes!("../../../assets/textures/horror/door.png");
pub const NOTE_TEXTURE: &[u8] = include_bytes!("../../../assets/textures/horror/note.png");
pub const LEVER_TEXTURE: &[u8] = include_bytes!("../../../assets/textures/horror/lever.png");

pub const ATMOSPHERE_AUDIO: &[u8] = include_bytes!("../../../assets/audio/horror/atmosphere.mp3");
pub const GENERATOR_AUDIO: &[u8] = include_bytes!("../../../assets/audio/horror/generator.mp3");
pub const RUBBLE_AUDIO: &[u8] = include_bytes!("../../../assets/audio/horror/rubble.mp3");
pub const MONSTER_AUDIO: &[u8] = include_bytes!("../../../assets/audio/horror/monster.mp3");
pub const FOOTSTEPS_AUDIO: &[u8] = include_bytes!("../../../assets/audio/horror/footsteps.mp3");
pub const DOOR_CREAK_AUDIO: &[u8] = include_bytes!("../../../assets/audio/horror/door_creak.mp3");

pub const GRAB_RANGE: f32 = 3.0;
pub const INTERACT_RANGE: f32 = 2.5;
pub const INTERACT_CONE_RADIUS: f32 = 40.0;
pub const MIN_GRAB_DISTANCE: f32 = 0.8;
pub const MAX_GRAB_DISTANCE: f32 = 3.0;
pub const SCROLL_DISTANCE_SPEED: f32 = 0.3;
pub const THROW_STRENGTH: f32 = 12.0;
pub const GRAB_STIFFNESS: f32 = 150.0;
pub const GRAB_DAMPING_RATIO: f32 = 1.0;
pub const MAX_GRAB_FORCE: f32 = 80.0;
pub const ANGULAR_DAMPING: f32 = 0.95;
pub const STANDING_CAMERA_HEIGHT: f32 = 0.8;
pub const CROUCHING_CAMERA_HEIGHT: f32 = 0.3;
pub const LEAN_AMOUNT: f32 = 0.4;
pub const LEAN_ANGLE: f32 = 0.15;
pub const LEAN_SPEED: f32 = 8.0;

pub const ROOM_HEIGHT: f32 = 3.0;
pub const WALL_THICKNESS: f32 = 0.3;
