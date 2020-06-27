use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

fn main() {
    App::build()
        .add_default_plugins()
        .add_resource(Scoreboard { score: 0 })
        .add_resource(ClearColor(Color::rgb(0.7, 0.7, 0.7)))
        .add_startup_system(setup.system())
        .add_system(paddle_movement_system.system())
        .add_system(ball_collision_system.system())
        .add_system(ball_movement_system.system())
        .add_system(scoreboard_system.system())
        .run();
}

struct Paddle {
    speed: f32,
}

struct Ball {
    velocity: Vec3,
}

struct Brick;
struct Wall;

struct Scoreboard {
    score: usize,
}

fn setup(
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    command_buffer: &mut CommandBuffer,
) {
    // Add the game's entities to our world
    let mut builder = command_buffer.build();
    builder
        // camera
        .entity_with(OrthographicCameraComponents::default())
        // paddle
        .entity_with(SpriteComponents {
            material: materials.add(Color::rgb(0.2, 0.2, 0.8).into()),
            translation: Translation(Vec3::new(0.0, -250.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(120.0, 30.0),
            },
            ..Default::default()
        })
        .with(Paddle { speed: 500.0 })
        // ball
        .entity_with(SpriteComponents {
            material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
            translation: Translation(Vec3::new(0.0, -100.0, 1.0)),
            sprite: Sprite {
                size: Vec2::new(30.0, 30.0),
            },
            ..Default::default()
        })
        .with(Ball {
            velocity: 400.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
        })
        // scoreboard
        .entity_with(LabelComponents {
            label: Label {
                font: asset_server.load("assets/fonts/FiraSans-Bold.ttf").unwrap(),
                text: "Score:".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8).into(),
                    font_size: 40.0,
                },
            },
            node: Node::new(Anchors::TOP_LEFT, Margins::new(10.0, 50.0, 10.0, 50.0)),
            ..Default::default()
        });

    // Add walls
    let wall_material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    let wall_thickness = 10.0;
    let bounds = Vec2::new(900.0, 600.0);

    builder
        // left
        .entity_with(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(-bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(wall_thickness, bounds.y() + wall_thickness),
            },
            ..Default::default()
        })
        .with(Wall)
        // right
        .entity_with(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(bounds.x() / 2.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(wall_thickness, bounds.y() + wall_thickness),
            },
            ..Default::default()
        })
        .with(Wall)
        // bottom
        .entity_with(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(0.0, -bounds.y() / 2.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(bounds.x() + wall_thickness, wall_thickness),
            },
            ..Default::default()
        })
        .with(Wall)
        // top
        .entity_with(SpriteComponents {
            material: wall_material,
            translation: Translation(Vec3::new(0.0, bounds.y() / 2.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(bounds.x() + wall_thickness, wall_thickness),
            },
            ..Default::default()
        })
        .with(Wall);

    // Add bricks
    let brick_rows = 4;
    let brick_columns = 5;
    let brick_spacing = 20.0;
    let brick_size = Vec2::new(150.0, 30.0);
    let bricks_width = brick_columns as f32 * (brick_size.x() + brick_spacing) - brick_spacing;
    // center the bricks and move them up a bit
    let bricks_offset = Vec3::new(-(bricks_width - brick_size.x()) / 2.0, 100.0, 0.0);

    for row in 0..brick_rows {
        let y_position = row as f32 * (brick_size.y() + brick_spacing);
        for column in 0..brick_columns {
            let brick_position = Vec3::new(
                column as f32 * (brick_size.x() + brick_spacing),
                y_position,
                0.0,
            ) + bricks_offset;
            builder
                // brick
                .entity_with(SpriteComponents {
                    material: materials.add(Color::rgb(0.2, 0.2, 0.8).into()),
                    sprite: Sprite { size: brick_size },
                    translation: Translation(brick_position),
                    ..Default::default()
                })
                .with(Brick);
        }
    }
}

fn paddle_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    world: &mut SubWorld,
    query: &mut Query<(Read<Paddle>, Write<Translation>)>,
) {
    for (paddle, mut translation) in query.iter_mut(world) {
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) {
            direction -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += 1.0;
        }

        *translation.0.x_mut() += time.delta_seconds * direction * paddle.speed;
    }
}

fn ball_movement_system(
    time: Res<Time>,
    world: &mut SubWorld,
    ball_query: &mut Query<(Read<Ball>, Write<Translation>)>,
) {
    for (ball, mut translation) in ball_query.iter_mut(world) {
        translation.0 += ball.velocity * time.delta_seconds;
    }
}

fn scoreboard_system(
    scoreboard: Res<Scoreboard>,
    world: &mut SubWorld,
    query: &mut Query<Write<Label>>,
) {
    for mut label in query.iter_mut(world) {
        label.text = format!("Score: {}", scoreboard.score);
    }
}

fn ball_collision_system(
    mut scoreboard: ResMut<Scoreboard>,
    command_buffer: &mut CommandBuffer,
    world: &mut SubWorld,
    ball_query: &mut Query<(Write<Ball>, Read<Translation>, Read<Sprite>)>,
    paddle_query: &mut Query<(Read<Paddle>, Read<Translation>, Read<Sprite>)>,
    brick_query: &mut Query<(Read<Brick>, Read<Translation>, Read<Sprite>)>,
    wall_query: &mut Query<(Read<Wall>, Read<Translation>, Read<Sprite>)>,
) {
    for (mut ball, translation, sprite) in ball_query.iter_mut(world) {
        let ball_position = translation.0;
        let ball_size = sprite.size;
        let velocity = &mut ball.velocity;
        let mut collision = None;

        // check collision with walls
        for (_wall, translation, sprite) in wall_query.iter(world) {
            if collision.is_some() {
                break;
            }

            collision = collide(ball_position, ball_size, translation.0, sprite.size);
        }

        // check collision with paddle(s)
        for (_paddle, translation, sprite) in paddle_query.iter(world) {
            if collision.is_some() {
                break;
            }

            collision = collide(ball_position, ball_size, translation.0, sprite.size);
        }

        // check collision with bricks
        for (brick_entity, (_brick, translation, sprite)) in brick_query.iter_entities(world) {
            if collision.is_some() {
                break;
            }

            collision = collide(ball_position, ball_size, translation.0, sprite.size);
            if collision.is_some() {
                scoreboard.score += 1;
                command_buffer.delete(brick_entity);
            }
        }

        // reflect the ball when it collides
        let mut reflect_x = false;
        let mut reflect_y = false;

        // only reflect if the ball's velocity is going in the opposite direction of the collision
        match collision {
            Some(Collision::Left) => reflect_x = velocity.x() > 0.0,
            Some(Collision::Right) => reflect_x = velocity.x() < 0.0,
            Some(Collision::Top) => reflect_y = velocity.y() < 0.0,
            Some(Collision::Bottom) => reflect_y = velocity.y() > 0.0,
            None => {}
        }

        // reflect velocity on the x-axis if we hit something on the x-axis
        if reflect_x {
            *velocity.x_mut() = -velocity.x();
        }

        // reflect velocity on the y-axis if we hit something on the y-axis
        if reflect_y {
            *velocity.y_mut() = -velocity.y();
        }
    }
}