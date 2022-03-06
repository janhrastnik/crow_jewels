use bevy::core::FixedTimestep;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin};
use rand::Rng;

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
    wing_audio_channel: AudioChannel,
    is_dead: bool,
}

#[derive(Component)]
struct Person {
    frame_index: usize,
}

const TIME_STEP: f32 = 1.0 / 60.0;

const SPAWN_STEP: f32 = 5.0;

#[derive(PartialEq)]
enum CrowState {
    Idle,
    Run,
    Fly,
}

#[derive(Component)]
struct GameOverUI;

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
    Person,
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
    crow_takeoff: Handle<TextureAtlas>,
}

fn game_not_over(crow_query: Query<&Crow>) -> ShouldRun {
    let crow = crow_query.single();
    if crow.is_dead {
        return ShouldRun::No;
    }
    ShouldRun::Yes
}

fn game_is_over(crow_query: Query<&Crow>) -> ShouldRun {
    let crow = crow_query.single();
    if crow.is_dead {
        return ShouldRun::Yes;
    }
    ShouldRun::No
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Crow Jewels".to_string(),
            width: 800.,
            height: 600.,
            mode: bevy::window::WindowMode::Windowed,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(spawn_background)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_run_criteria(game_not_over)
                .with_system(crow_input)
                .with_system(animate_crow)
                .with_system(collision_check)
                .with_system(move_people)
                .with_system(animate_people)
                .with_system(ui),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(SPAWN_STEP as f64))
                .with_system(spawn_jewel),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(game_is_over)
                .with_system(gameover_screen),
        )
        .run();
}

fn animate_people(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut people_query: Query<(
        &mut Person,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut person, mut timer, mut sprite, texture_atlas_handle) in people_query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
            person.frame_index = 0;
        }
    }
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

fn spawn_jewel(mut commands: Commands, asset_server: Res<AssetServer>) {
    let num: f32 = rand::thread_rng().gen_range(-1500..1500) as f32;
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("ring.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(64.0, 64.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(num, 20.0, 1.0),
            ..Default::default()
        })
        .insert(Collider {
            width: 64.0,
            height: 64.0,
            collider_type: ColliderType::Jewel,
        });
}

fn spawn_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    audio: Res<Audio>,
) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(BirdCamera {});
    commands.spawn_bundle(UiCameraBundle::default());
    let background_handle = asset_server.load("sky.png");
    commands
        .spawn_bundle(SpriteBundle {
            texture: background_handle,
            transform: Transform::from_scale(Vec3::new(12.0, 12.0, 0.0)),
            ..Default::default()
        })
        .insert(Background {});
    // load sprites into resources
    let idle_handle = asset_server.load("crow.png");
    let idle_atlas = TextureAtlas::from_grid(idle_handle, Vec2::new(96.0, 96.0), 11, 1);
    let crow_idle_handle = texture_atlases.add(idle_atlas);

    let run_handle = asset_server.load("crowrun.png");
    let run_atlas = TextureAtlas::from_grid(run_handle, Vec2::new(96.0, 96.0), 7, 1);
    let crow_run_handle = texture_atlases.add(run_atlas);

    let takeoff_handle = asset_server.load("crow_takeoff2x.png");
    let takeoff_atlas = TextureAtlas::from_grid(takeoff_handle, Vec2::new(134.0, 134.0), 6, 1);
    let crow_takeoff_handle = texture_atlases.add(takeoff_atlas);

    let person_handle = asset_server.load("walking_stickman.png");
    let person_atlas = TextureAtlas::from_grid(person_handle, Vec2::new(80.0, 80.0), 4, 1);
    let person_handle = texture_atlases.add(person_atlas);

    let sprites = Sprites {
        crow_idle: crow_idle_handle.clone(),
        crow_run: crow_run_handle,
        crow_takeoff: crow_takeoff_handle,
    };

    commands.insert_resource(sprites);

    // spawn static entities
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("brick.png"),
            transform: Transform::from_xyz(100.0, 16.0, 1.0),
            ..Default::default()
        })
        .insert(Collider {
            width: 32.0,
            height: 32.0,
            collider_type: ColliderType::Surface,
        });
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("brick.png"),
            transform: Transform::from_xyz(0.0, 50.0, 1.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider {
            width: 100.0,
            height: 100.0,
            collider_type: ColliderType::Surface,
        });
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("brick.png"),
            transform: Transform::from_xyz(700.0, 50.0, 1.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(32.0, 100.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider {
            width: 32.0,
            height: 100.0,
            collider_type: ColliderType::Surface,
        });
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("vines.png"),
            transform: Transform::from_xyz(-700.0, 50.0, 1.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(32.0, 100.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider {
            width: 32.0,
            height: 100.0,
            collider_type: ColliderType::Surface,
        });
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("vines.png"),
            transform: Transform::from_xyz(-1150.0, 50.0, 1.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(32.0, 100.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Collider {
            width: 32.0,
            height: 100.0,
            collider_type: ColliderType::Surface,
        });

    // floor
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("dirtfloor.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1000.0, 1000.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(-1000.0, -500.0, 1.0),
            ..Default::default()
        })
        .insert(Collider {
            width: 1000.0,
            height: 1000.0,
            collider_type: ColliderType::Surface,
        });
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("dirtfloor.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1000.0, 1000.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, -500.0, 1.0),
            ..Default::default()
        })
        .insert(Collider {
            width: 1000.0,
            height: 1000.0,
            collider_type: ColliderType::Surface,
        });
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("dirtfloor.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1000.0, 1000.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(1000.0, -500.0, 1.0),
            ..Default::default()
        })
        .insert(Collider {
            width: 1000.0,
            height: 1000.0,
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

    // spawn people
    for _ in 0..10 {
        let mut rng = rand::thread_rng();
        let mut num: f32 = rng.gen_range(300..1500) as f32;
        let sign: bool = rng.gen_bool(0.5);
        if sign {
            num *= -1.0;
        }
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: person_handle.clone(),
                transform: Transform::from_xyz(num, 20.0, 1.0),
                ..Default::default()
            })
            .insert(Collider {
                width: 64.0,
                height: 64.0,
                collider_type: ColliderType::Person,
            })
            .insert(Person { frame_index: 0 })
            .insert(AnimationTimer(Timer::from_seconds(0.1, true)));
    }
    // spawn the crow
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: crow_idle_handle,
            transform: Transform::from_xyz(0.0, 150.0, 1.0),
            ..Default::default()
        })
        .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
        .insert(Crow {
            crow_state: CrowState::Idle,
            acceleration: 0.0,
            idle_frame_tick_times: vec![10, 1, 1, 1, 1, 1, 1, 1, 2, 10, 10],
            fly_frame_tick_times: vec![1, 1, 1, 1, 1, 1],
            run_frame_tick_times: vec![1, 1, 1, 1, 1, 1, 1, 1, 1],
            idle_frame_tick_counter: 0,
            is_colliding_vert: IsColliding::No,
            is_colliding_hori: IsColliding::No,
            score: 0,
            wing_audio_channel: AudioChannel::new("wings".to_owned()),
            is_dead: false,
        });

    let font = asset_server.load("Inconsolata-Regular.ttf");
    // text, ui
    commands.spawn_bundle(Text2dBundle {
        transform: Transform::from_xyz(-150.0, 200.0, 1.0),
        text: Text::with_section(
            "Steal the jewelry!".to_string(),
            TextStyle {
                font: font.clone(),
                font_size: 40.0,
                color: Color::BLACK,
            },
            Default::default(),
        ),
        ..Default::default()
    });
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
                    color: Color::BLACK,
                },
                Default::default(),
            ),
            ..Default::default()
        })
        .insert(ScoreText);
    audio.play_looped(asset_server.load("AcesHighKevinMacleod.ogg"));
    audio.set_volume(0.3);
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
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
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
) {
    let (mut camera_transform, _) = camera_query.single_mut();
    let (mut background_transform, _, _) = background_query.single_mut();
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
        audio.play_in_channel(asset_server.load("wingflap.wav"), &crow.wing_audio_channel);
        transform.translation.y += 10.0;
        if crow.crow_state != CrowState::Fly {
            crow.crow_state = CrowState::Fly;
            sprite.index = 0;
            *crow_handle = sprites.crow_takeoff.clone();
        }
    }
    if keyboard_input.pressed(KeyCode::Left)
        && crow.is_colliding_hori != IsColliding::Left
        && transform.translation.x > -1500.0
    {
        transform.translation.x += -200.0 * time.delta_seconds();
        sprite.flip_x = true;
        if crow.crow_state == CrowState::Idle {
            crow.crow_state = CrowState::Run;
            sprite.index = 0;
            *crow_handle = sprites.crow_run.clone();
        }
    } else if keyboard_input.pressed(KeyCode::Right)
        && crow.is_colliding_hori != IsColliding::Right
        && transform.translation.x < 1500.0
    {
        transform.translation.x += 200.0 * time.delta_seconds();
        sprite.flip_x = false;
        if crow.crow_state == CrowState::Idle {
            crow.crow_state = CrowState::Run;
            sprite.index = 0;
            *crow_handle = sprites.crow_run.clone();
        }
    } else if crow.crow_state != CrowState::Fly {
        crow.crow_state = CrowState::Idle;
        *crow_handle = sprites.crow_idle.clone();
    }

    crow.acceleration -= 5.0;

    camera_transform.translation = transform.translation;
    background_transform.translation =
        Vec3::new(transform.translation.x, transform.translation.y, 0.0);
}

fn move_people(
    time: Res<Time>,
    mut people_query: Query<(&Person, &mut Transform, &mut TextureAtlasSprite)>,
    mut crow_query: Query<(&Crow, &Transform, Without<Person>)>,
) {
    let (_, crow_transform, _) = crow_query.single_mut();
    for (_, mut person_transform, mut sprite) in people_query.iter_mut() {
        if (crow_transform.translation.x - person_transform.translation.x).abs() < 350.0 {
            if crow_transform.translation.x > person_transform.translation.x {
                sprite.flip_x = false;
                person_transform.translation.x += 25.0 * time.delta_seconds();
            } else {
                sprite.flip_x = true;
                person_transform.translation.x -= 25.0 * time.delta_seconds();
            }
        }
    }
}

fn collision_check(
    mut commands: Commands,
    mut crow_query: Query<(&mut Crow, &Transform)>,
    collider_query: Query<(Entity, &Collider, &Transform)>,
    mut score_query: Query<(Entity, &mut Text, With<ScoreText>)>,
    asset_server: Res<AssetServer>,
) {
    let (mut crow, crow_transform) = crow_query.single_mut();
    let (score_entity, _, _) = score_query.single_mut();
    let mut found_collision = false;
    for (entity, collider, collider_transform) in collider_query.iter() {
        let collision = collide(
            collider_transform.translation,
            Vec2::new(collider.width, collider.height),
            crow_transform.translation,
            Vec2::new(60.0, 60.0),
        );

        if let Some(collision) = collision {
            found_collision = true;

            if collider.collider_type == ColliderType::Jewel {
                crow.score += 1;
                commands.entity(entity).despawn();
            } else if collider.collider_type == ColliderType::Person {
                commands
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            justify_content: JustifyContent::SpaceBetween,
                            ..Default::default()
                        },
                        color: Color::RED.into(),
                        ..Default::default()
                    })
                    .insert(GameOverUI {});
                commands.entity(score_entity).despawn();
                crow.is_dead = true;
                let font = asset_server.load("Inconsolata-Regular.ttf");
                commands
                    .spawn_bundle(TextBundle {
                        style: Style {
                            align_self: AlignSelf::Center,
                            position_type: PositionType::Absolute,
                            ..Default::default()
                        },
                        text: Text::with_section(
                            "        Game Over\n    Press [Space] to restart".to_string(),
                            TextStyle {
                                font,
                                font_size: 30.0,
                                color: Color::BLACK,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    })
                    .insert(GameOverUI {});
            }

            match collision {
                Collision::Left => crow.is_colliding_hori = IsColliding::Left,
                Collision::Right => crow.is_colliding_hori = IsColliding::Right,
                Collision::Top => crow.is_colliding_vert = IsColliding::Top,
                Collision::Bottom => crow.is_colliding_vert = IsColliding::Bottom,
            };
        } else if !found_collision {
            crow.is_colliding_hori = IsColliding::No;
            crow.is_colliding_vert = IsColliding::No;
        }
    }
}

fn ui(mut score_query: Query<(&mut Text, With<ScoreText>)>, mut crow_query: Query<&Crow>) {
    let crow = crow_query.single_mut();
    let (mut score, _) = score_query.single_mut();
    score.sections[0].value = format!("Score: {}", crow.score);
}

fn gameover_screen(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut gameover_ui_query: Query<(Entity, &GameOverUI)>,
    mut crow_query: Query<(&mut Crow, &mut Transform)>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        for (entity, _) in gameover_ui_query.iter_mut() {
            commands.entity(entity).despawn();
        }
        let (mut crow, mut crow_transform) = crow_query.single_mut();
        crow.is_dead = false;
        crow.score = 0;
        crow_transform.translation = Vec3::new(0.0, 150.0, 1.0);
        let font = asset_server.load("Inconsolata-Regular.ttf");
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
                        color: Color::BLACK,
                    },
                    Default::default(),
                ),
                ..Default::default()
            })
            .insert(ScoreText);
    }
}
