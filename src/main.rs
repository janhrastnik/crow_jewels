use bevy::prelude::*;

#[derive(Component)]
struct Crow {
    crow_state: CrowState,
    acceleration: f32,
    idle_frame_tick_times: Vec<usize>,
    fly_frame_tick_times: Vec<usize>,
    idle_frame_tick_counter: usize,
}

enum CrowState {
    Idle,
    Fly,
}

#[derive(Component)]
struct AnimationTimer(Timer);

#[derive(Default)]
struct Sprites {
    // sprites that are meant to be reused
    crow_idle: Handle<TextureAtlas>,
    crow_fly: Handle<TextureAtlas>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(crow_input)
        .add_system(animate_sprite)
        .run();
}

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut Crow,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut crow, mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            crow.idle_frame_tick_counter += 1;
            match crow.crow_state {
                CrowState::Idle => {
                    if crow.idle_frame_tick_counter > crow.idle_frame_tick_times[sprite.index] {
                        let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
                        sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
                        crow.idle_frame_tick_counter = 0;
                    }
                }
                _ => {
                    if crow.idle_frame_tick_counter > crow.fly_frame_tick_times[sprite.index] {
                        let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
                        sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
                        crow.idle_frame_tick_counter = 0;
                    }
                }
            }
        }
    }
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // load sprites into resources
    let texture_handle = asset_server.load("crow.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(96.0, 96.0), 11, 1);
    let crow_idle_handle = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load("crowfly.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(96.0, 96.0), 7, 1);
    let crow_fly_handle = texture_atlases.add(texture_atlas);

    commands.insert_resource(Sprites {
        crow_idle: crow_idle_handle.clone(),
        crow_fly: crow_fly_handle,
    });

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    /*
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("earf.gif"),
        ..Default::default()
    });
    */
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: crow_idle_handle,
            transform: Transform::from_xyz(0.0, 100.0, 0.0),
            ..Default::default()
        })
        .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
        .insert(Crow {
            crow_state: CrowState::Idle,
            acceleration: 0.0,
            idle_frame_tick_times: vec![10, 1, 1, 1, 1, 1, 1, 1, 2, 10, 10],
            fly_frame_tick_times: vec![1, 1, 1, 1, 1, 1, 1, 1, 1],
            idle_frame_tick_counter: 0,
        });
}

fn crow_input(
    time: Res<Time>,
    sprites: Res<Sprites>,
    keyboard_input: Res<Input<KeyCode>>,
    mut crow_transform: Query<(
        &mut Crow,
        &mut Transform,
        &mut Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut crow, mut transform, mut crow_handle, mut sprite) in crow_transform.iter_mut() {
        if keyboard_input.pressed(KeyCode::Space) {
            crow.acceleration = 200.0;
            crow.crow_state = CrowState::Fly;

            sprite.index = 0;
            *crow_handle = sprites.crow_fly.clone();
        }
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x += -200.0 * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += 200.0 * time.delta_seconds();
        }

        transform.translation.y += 1.0 * crow.acceleration * time.delta_seconds();
        if transform.translation.y > 0.0 {
            crow.acceleration -= 5.0;
        } else {
            crow.crow_state = CrowState::Idle;
            *crow_handle = sprites.crow_idle.clone();
            crow.acceleration = 0.0;
        }
    }
}
