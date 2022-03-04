use bevy::core::FixedTimestep;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};

#[derive(Component)]
struct Crow {
    crow_state: CrowState,
    acceleration: f32,
    idle_frame_tick_times: Vec<usize>,
    fly_frame_tick_times: Vec<usize>,
    run_frame_tick_times: Vec<usize>,
    idle_frame_tick_counter: usize,
    is_colliding_vert: IsColliding,
    is_colliding_hori: IsColliding,
    score: usize,
}

const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(PartialEq)]
enum CrowState {
    Idle,
    Run,
    Fly,
}

#[derive(Component)]
struct DebugText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct Collider {
    width: f32,
    height: f32,
    collider_type: ColliderType,
}

#[derive(PartialEq)]
enum ColliderType {
    Surface,
    Jewel,
}

#[derive(Component)]
struct BirdCamera {}

#[derive(PartialEq, Debug)]
enum IsColliding {
    Top,
    Bottom,
    Left,
    Right,
    No,
}

#[derive(Component)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Background();

#[derive(Default)]
struct Sprites {
    // sprites that are meant to be reused
    crow_idle: Handle<TextureAtlas>,
    crow_run: Handle<TextureAtlas>,
    crow_fly: Handle<TextureAtlas>,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Crow Jewels".to_string(),
            width: 400.,
            height: 400.,
            mode: bevy::window::WindowMode::Windowed,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(spawn_background)
        .add_system(ui)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(crow_input)
                .with_system(animate_crow)
                .with_system(collision_check),
        )
        .run();
}

fn animate_crow(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut crow_query: Query<(
        &mut Crow,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    let (mut crow, mut timer, mut sprite, texture_atlas_handle) = crow_query.single_mut();
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

            CrowState::Run => {
                if crow.idle_frame_tick_counter > crow.run_frame_tick_times[sprite.index] {
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

fn spawn_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(BirdCamera {});
    commands.spawn_bundle(UiCameraBundle::default());
    let background_handle = asset_server.load("sky.png");
    commands
        .spawn_bundle(SpriteBundle {
            texture: background_handle,
            transform: Transform::from_scale(Vec3::new(10.0, 10.0, 0.0)),
            ..Default::default()
        })
        .insert(Background {});
    // load sprites into resources
    let idle_handle = asset_server.load("crow.png");
    let idle_atlas = TextureAtlas::from_grid(idle_handle, Vec2::new(96.0, 96.0), 11, 1);
    let crow_idle_handle = texture_atlases.add(idle_atlas);

    let fly_handle = asset_server.load("crowfly.png");
    let fly_atlas = TextureAtlas::from_grid(fly_handle, Vec2::new(96.0, 96.0), 7, 1);
    let crow_fly_handle = texture_atlases.add(fly_atlas);

    let run_handle = asset_server.load("crowrun.png");
    let run_atlas = TextureAtlas::from_grid(run_handle, Vec2::new(96.0, 96.0), 7, 1);
    let crow_run_handle = texture_atlases.add(run_atlas);

    let sprites = Sprites {
        crow_idle: crow_idle_handle.clone(),
        crow_run: crow_run_handle,
        crow_fly: crow_fly_handle,
    };

    commands.insert_resource(sprites);

    // spawn static entities
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("brick.png"),
            transform: Transform::from_xyz(100.0, 10.0, 1.0),
            ..Default::default()
        })
        .insert(Collider {
            width: 32.0,
            height: 32.0,
            collider_type: ColliderType::Surface,
        });

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("dirtfloor.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(300.0, 300.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(-300.0, -150.0, 1.0),
            ..Default::default()
        })
        .insert(Collider {
            width: 300.0,
            height: 300.0,
            collider_type: ColliderType::Surface,
        });
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("dirtfloor.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(300.0, 300.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, -150.0, 1.0),
            ..Default::default()
        })
        .insert(Collider {
            width: 300.0,
            height: 300.0,
            collider_type: ColliderType::Surface,
        });
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("ring.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(64.0, 64.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(-150.0, 20.0, 1.0),
            ..Default::default()
        })
        .insert(Collider {
            width: 64.0,
            height: 64.0,
            collider_type: ColliderType::Jewel,
        });

    // spawn the crow
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: crow_idle_handle,
            transform: Transform::from_xyz(0.0, 100.0, 1.0),
            ..Default::default()
        })
        .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
        .insert(Crow {
            crow_state: CrowState::Idle,
            acceleration: 0.0,
            idle_frame_tick_times: vec![10, 1, 1, 1, 1, 1, 1, 1, 2, 10, 10],
            fly_frame_tick_times: vec![1, 1, 1, 1, 1, 1, 1, 1, 1],
            run_frame_tick_times: vec![1, 1, 1, 1, 1, 1, 1, 1, 1],
            idle_frame_tick_counter: 0,
            is_colliding_vert: IsColliding::No,
            is_colliding_hori: IsColliding::No,
            score: 0,
        });

    let font = asset_server.load("Inconsolata-Regular.ttf");

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                size: Size {
                    width: Val::Px(200.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "Score".to_string(),
                TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    color: Color::WHITE,
                },
                Default::default(),
            ),
            ..Default::default()
        })
        .insert(DebugText);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                size: Size {
                    width: Val::Px(200.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "Score: 0".to_string(),
                TextStyle {
                    font,
                    font_size: 50.0,
                    color: Color::WHITE,
                },
                Default::default(),
            ),
            ..Default::default()
        })
        .insert(ScoreText);
}

fn crow_input(
    time: Res<Time>,
    sprites: Res<Sprites>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<(&mut Transform, &BirdCamera)>,
    mut background_query: Query<(&mut Transform, &Background, Without<BirdCamera>)>,
    mut crow_query: Query<(
        &mut Crow,
        &mut Transform,
        Without<BirdCamera>,
        Without<Background>,
        &mut Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
    )>,
) {
    let (mut camera_transform, mut camera) = camera_query.single_mut();
    let (mut background_transform, mut background, _) = background_query.single_mut();
    let (mut crow, mut transform, _, _, mut crow_handle, mut sprite) = crow_query.single_mut();
    if crow.is_colliding_vert != IsColliding::Bottom {
        transform.translation.y += 1.0 * crow.acceleration * time.delta_seconds();
    } else {
        if crow.crow_state == CrowState::Fly {
            crow.crow_state = CrowState::Idle;
            *crow_handle = sprites.crow_idle.clone();
            sprite.index = 0;
        }
        crow.acceleration = 0.0;
    }

    if keyboard_input.pressed(KeyCode::Space) {
        crow.acceleration = 200.0;
        transform.translation.y += 10.0;
        if crow.crow_state != CrowState::Fly {
            crow.crow_state = CrowState::Fly;
            sprite.index = 0;
            *crow_handle = sprites.crow_fly.clone();
        }
    }
    if keyboard_input.pressed(KeyCode::Left) && crow.is_colliding_hori != IsColliding::Left {
        transform.translation.x += -200.0 * time.delta_seconds();
        sprite.flip_x = true;
        if crow.crow_state == CrowState::Idle {
            crow.crow_state = CrowState::Run;
            sprite.index = 0;
            *crow_handle = sprites.crow_run.clone();
        }
    } else if keyboard_input.pressed(KeyCode::Right) && crow.is_colliding_hori != IsColliding::Right
    {
        println!("this runs");
        transform.translation.x += 200.0 * time.delta_seconds();
        sprite.flip_x = false;
        if crow.crow_state == CrowState::Idle {
            crow.crow_state = CrowState::Run;
            sprite.index = 0;
            *crow_handle = sprites.crow_run.clone();
        }
    } else {
        if crow.crow_state != CrowState::Fly {
            crow.crow_state = CrowState::Idle;
            *crow_handle = sprites.crow_idle.clone();
        }
    }
    crow.acceleration -= 5.0;

    camera_transform.translation = transform.translation;
    background_transform.translation =
        Vec3::new(transform.translation.x, transform.translation.y, 0.0);
}

fn collision_check(
    mut commands: Commands,
    mut crow_query: Query<(&mut Crow, &Transform)>,
    collider_query: Query<(Entity, &Collider, &Transform)>,
) {
    let (mut crow, crow_transform) = crow_query.single_mut();
    let mut found_collision = false;
    for (entity, collider, collider_transform) in collider_query.iter() {
        let collision = collide(
            collider_transform.translation,
            Vec2::new(collider.width, collider.height),
            crow_transform.translation,
            Vec2::new(60.0, 60.0),
        );

        if let Some(collision) = collision {
            println!("collision!");
            found_collision = true;

            if collider.collider_type == ColliderType::Jewel {
                crow.score += 1;
                commands.entity(entity).despawn();
            }

            match collision {
                Collision::Left => crow.is_colliding_hori = IsColliding::Left,
                Collision::Right => crow.is_colliding_hori = IsColliding::Right,
                Collision::Top => crow.is_colliding_vert = IsColliding::Top,
                Collision::Bottom => crow.is_colliding_vert = IsColliding::Bottom,
            };
        } else {
            if !found_collision {
                crow.is_colliding_hori = IsColliding::No;
                crow.is_colliding_vert = IsColliding::No;
            }
        }
    }
}

fn ui(
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<DebugText>>,
    mut score_query: Query<(&mut Text, With<ScoreText>, Without<DebugText>)>,
    mut crow_query: Query<(&Transform, &Crow)>,
) {
    let (transform, crow) = crow_query.single_mut();
    let (mut score, _, _) = score_query.single_mut();
    score.sections[0].value = format!("Score: {}", crow.score);
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("{}", transform.scale);
    }
}
