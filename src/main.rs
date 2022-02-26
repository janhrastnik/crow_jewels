use bevy::prelude::*;

#[derive(Component)]
struct Crow {
    acceleration: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(crow_input)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    /*
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("earf.gif"),
        ..Default::default()
    });
    */
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 100., 0.),
            ..Default::default()
        })
        .insert(Crow { acceleration: 0.0 });
}

fn crow_input(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut crow_transform: Query<(&mut Crow, &mut Transform)>,
) {
    for (mut crow, mut transform) in crow_transform.iter_mut() {
        if keyboard_input.pressed(KeyCode::Space) {
            crow.acceleration = 200.0;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x += -200.0 * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += 200.0 * time.delta_seconds();
        }

        transform.translation.y += 1.0 * crow.acceleration * time.delta_seconds();
        if transform.translation.y > 0.0 && crow.acceleration > -500.0 {
            crow.acceleration -= 5.0;
        } else {
            crow.acceleration = 0.0;
        }
    }
}
