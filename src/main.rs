use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    time::{FixedTimestep},
    text::Text2dBundle,
    // For debugging
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
};

// Set to 60 frames per second
const TIMESTEP: f32 = 3.0 / 60.0;

const PLAYER_SPEED: f32 = 250.0;


// Walls settings
const WALL_THICKNESS: f32 = 10.0;
const TOP_WALL: f32 = 540.0;
const LEFT_WALL: f32 = -960.0;
const BOTTOM_WALL: f32 = -540.0;
const RIGHT_WALL: f32 = 960.0;
const WALL_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);



// Main loop
fn main() {    
    App::new()
	.add_plugins(DefaultPlugins.set(WindowPlugin {
	    window: WindowDescriptor {
		title: "Boar Game".into(),
		width: 1024.0,
		height: 762.0,
		..default()
	    },
	    ..default()
	}))
	// Show framerate in console
	.add_plugin(LogDiagnosticsPlugin::default())
	.add_plugin(FrameTimeDiagnosticsPlugin::default())
	.add_startup_system(setup)
	.add_system_set(SystemSet::new()
			.with_run_criteria(FixedTimestep::step(TIMESTEP as f64))
			.with_system(move_player)
			.with_system(move_camera)
			.with_system(check_for_collisions))
	.add_system(bevy::window::close_on_esc)
	.run();
}


// Components for the characters
#[derive(Component)]
struct Player;

#[derive(Component)]
enum Npc {
    House,
    Boar,
}

#[derive(Component)]
struct HealthPoints(f32);

#[derive(Component)]
struct Name;


// Identifiers for cameras
#[derive(Component)]
struct MapCamera;

// Components to handle collisions
#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;


// Walls are a bundle consisting of a sprite and a collider
#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

// To better manipulate the walls, we will consider the four separately
enum WallLocation {
    Top,
    Left,
    Bottom,
    Right,
}

impl WallLocation {
    // Outputs the location of the (middle) of a wall
    fn position(&self) -> Vec2 {
	match self {
	    | WallLocation::Top => Vec2::new(0., TOP_WALL),
	    | WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
	    | WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
	    | WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
	}
    }

    // Outputs the length of the wall
    fn size(&self) -> Vec2 {
	match self {
	    WallLocation::Left | WallLocation::Right => Vec2::new(WALL_THICKNESS, TOP_WALL - BOTTOM_WALL - WALL_THICKNESS),
	    WallLocation::Top | WallLocation::Bottom => Vec2::new(RIGHT_WALL - LEFT_WALL - WALL_THICKNESS, WALL_THICKNESS),
	}
    }
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
	WallBundle {
	    sprite_bundle: SpriteBundle{
		transform: Transform{
		    // Not sure why we need to transform into Vec3 ??
		    translation: location.position().extend(0.0),
		    scale: location.size().extend(1.0),
		    ..default()
		},
		sprite: Sprite {
		    color: WALL_COLOR,
		    ..default()
		},
		..default()
	    },

	    collider: Collider,
	}
    }
}




// setup function that places everything in the World before the game starts
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Utilities
    commands.spawn((
	Camera2dBundle {
	    projection: OrthographicProjection {
		scale: 0.5,
		..default()
	    },
	    transform: Transform::from_xyz(350.0, 350.0, 0.5),
	    ..default()
	},
	MapCamera,
    ));

    // Background
    commands.spawn(SpriteBundle {
	texture: asset_server.load("sprites/background.png").into(),
	..default()
	});

	
    // Player character
    commands.spawn((
	SpriteBundle {
	    texture: asset_server.load("sprites/triangulus.png").into(),
	    transform: Transform::from_xyz(350., 350., 0.2),
	    ..default()
	},
	Player,
	HealthPoints(100.0),
	Collider,
    ));

    
    // House
    commands.spawn((
	SpriteBundle {
	    texture: asset_server.load("sprites/maison.png").into(),
	    transform: Transform::from_xyz(350.0, 0.0, 0.1),
	    ..default()
	},
	Npc::House,
    ));

    
    // Boar
    commands.spawn((
	SpriteBundle {
	    texture: asset_server.load("sprites/frank.png").into(),
	    transform: Transform::from_xyz(-254.0, 180.0, 0.1),
	    ..default()
	},
	Npc::Boar,
	HealthPoints(40.0),
    ));
    
    // Spawn the walls
    commands.spawn(WallBundle::new(WallLocation::Top));
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Right));  
}


// System to move the player sprite
fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query_player: Query<&mut Transform, With<Player>>,
) {
    let mut player_transform = query_player.single_mut();
    let mut x_direction = 0.0;
    let mut y_direction = 0.0;
    
    if keyboard_input.pressed(KeyCode::A){
	x_direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D){
	x_direction += 1.0;
    }
    if keyboard_input.pressed(KeyCode::W){
	y_direction += 1.0;
    }
    if keyboard_input.pressed(KeyCode::S){
	y_direction -= 1.0;
    }

    // Compute the new coordinates of Player
    let new_transform_x = player_transform.translation.x + x_direction * PLAYER_SPEED * TIMESTEP;
    let new_transform_y = player_transform.translation.y + y_direction * PLAYER_SPEED * TIMESTEP;

    // Bounds ensure that the sprite never goes out of the screen
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + 16.0;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - 16.0;
    let top_bound = TOP_WALL - WALL_THICKNESS / 2.0 - 24.0;
    let bottom_bound = BOTTOM_WALL + WALL_THICKNESS / 2.0 + 16.0;

    // Apply the translation
    player_transform.translation.x = new_transform_x.clamp(left_bound, right_bound);
    player_transform.translation.y = new_transform_y.clamp(bottom_bound, top_bound);
}


// System to move the camera sprite (following the player sprite)
fn move_camera (
    keyboard_input: Res<Input<KeyCode>>,
    mut query_camera: Query<&mut Transform, With<MapCamera>>,
) {
    let mut camera_transform = query_camera.single_mut();
    let mut x_direction = 0.0;
    let mut y_direction = 0.0;
    
    if keyboard_input.pressed(KeyCode::A){
	x_direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D){
	x_direction += 1.0;
    }
    if keyboard_input.pressed(KeyCode::W){
	y_direction += 1.0;
    }
    if keyboard_input.pressed(KeyCode::S){
	y_direction -= 1.0;
    }

    // Compute the new coordinates of Player
    let new_transform_x = camera_transform.translation.x + x_direction * PLAYER_SPEED * TIMESTEP;
    let new_transform_y = camera_transform.translation.y + y_direction * PLAYER_SPEED * TIMESTEP;

    // Bounds ensure that the sprite never goes out of the screen
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + 16.0;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - 16.0;
    let top_bound = TOP_WALL - WALL_THICKNESS / 2.0 - 24.0;
    let bottom_bound = BOTTOM_WALL + WALL_THICKNESS / 2.0 + 16.0;

    // Apply the translation
    camera_transform.translation.x = new_transform_x.clamp(left_bound, right_bound);
    camera_transform.translation.y = new_transform_y.clamp(bottom_bound, top_bound);
}



// System to handle collision events (interactions between two sprites)
// (Not working)
fn check_for_collisions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<&mut Transform, With<Player>>,
    collider_query: Query<(&Transform, Option<&Npc>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let player_transform = player_query.single_mut();

    for (transform, maybe_npc) in &collider_query {
	let collision = collide(
	    player_transform.translation,   // Location of first object involved in collision (player)
	    Vec2::new(64.0, 64.0),          // Size of first object involved in collision (player)
	    transform.translation,          // Location of second object involved in collision
	    Vec2::new(64.0, 64.0),          // Size of second object involved in collision
	);
    
	
	if let Some(collision) = collision {
	    // Send the a signal to other systems so they can react
	    collision_events.send_default();
	    
	    if let Some(npc) = maybe_npc {
		match npc {
		    Npc::House => {
			let font = asset_server.load("fonts/FiraMono-Medium.ttf");
			let text_style = TextStyle {
			    font: font.clone(),
			    font_size: 18.0,
			    color: Color::GREEN,
			    ..default()
			};
			let text_alignment = TextAlignment::CENTER;

			commands.spawn(Text2dBundle {
			    text: Text::from_section("House", text_style.clone())
				.with_alignment(text_alignment),
			    transform: Transform::from_xyz(350.0, 0.0, 0.2),
			    ..default()
			});
		    },
		    Npc::Boar => todo!(),
		}
	    }
	}
    }
}
