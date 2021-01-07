use bevy::prelude::*;
use rand::prelude::random;
use std::{any::type_name, time::Duration};

const ARENA_WIDTH: u32 = 12;
const ARENA_HEIGHT: u32 = 12;
const FOOD_SPAWN_TIME: u64 = 1000;


struct GameOverEvent;

enum Games {
    QLearn,
    DeepLearn,
    Human
}
struct GameMode {
    value: Games
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}


#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Position { x: i32, y: i32 }
impl Position {
    pub fn generate() -> Self {
        Self {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        }
    }
}

struct Size { width: f32, height: f32 }
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}


#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}
#[derive(Debug, Default)]
struct WindowSize { height: f32, width: f32 }

struct SnakeHead { direction: Direction }
struct SnakeMaterials {
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>
}

impl FromResources for SnakeMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        let asset_server = resources.get::<AssetServer>().unwrap();
        let head: Handle<Texture> = asset_server.load("sprites/head.png");
        let apple: Handle<Texture> = asset_server.load("sprites/apple.png");
        SnakeMaterials {
            segment_material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
            food_material: materials.add(apple.into()),
            head_material: materials.add(head.into()),
        }
    }
}

struct SnakeSegment;
#[derive(Default)]
struct SnakeSegments(Vec<Entity>);

fn spawn_segment(
    commands: &mut Commands,
    material: &Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn(SpriteBundle {
            material: material.clone(),
            ..Default::default()
        })
        .with(SnakeSegment)
        .with(position)
        .with(Size::square(0.65))
        .current_entity()
        .unwrap()
}

struct SnakeMoveTimer(Timer);

struct Food;

struct FoodSpawnTimer(Timer);
impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(FOOD_SPAWN_TIME), true))
    }
}

fn get_positions(
    segments: ResMut<SnakeSegments>,
    mut positions: Query<&mut Position>
) -> Vec<Position> {
    return segments.0
        .iter()
        .map(|e| *positions.get_mut(*e).unwrap())
        .collect::<Vec<Position>>()
}

fn food_spawner(
    commands: &mut Commands,
    materials: Res<SnakeMaterials>,
    time: Res<Time>,
    segments: ResMut<SnakeSegments>,
    mut timer: Local<FoodSpawnTimer>,
    positions: Query<&mut Position>
) {
    if timer.0.tick(time.delta_seconds()).finished() {
        let segment_positions = get_positions(segments, positions);
        let mut food_position = Position::generate();

        while segment_positions.iter().any(|&segment| segment == food_position){
            food_position = Position::generate();
        };

        commands
            .spawn(SpriteBundle {
                material: materials.food_material.clone(),
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, -1.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .with(food_position)
            .with(Food)
            .with(Size::square(0.8));
    }
}

fn setup(
    commands: &mut Commands,
    // asset_server: Res<AssetServer>,
    mut _materials: ResMut<Assets<ColorMaterial>>
){
    commands
        .spawn(CameraUiBundle::default())
        .spawn(Camera2dBundle::default());
}

fn spawn_snake(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    materials: Res<SnakeMaterials>,
    mut segments: ResMut<SnakeSegments>,
) {
    commands.spawn(TextBundle {
        text: Text {
            value: "0".to_string(),
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            style: TextStyle {
                font_size: 100.0,
                color: Color::WHITE,
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
        },
        ..Default::default()
    }).with(Score(0));

    segments.0 = vec![
        commands
            .spawn(SpriteBundle {
                material: materials.head_material.clone(),
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, -1.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .with(SnakeHead {
                direction: Direction::Up,
            })
            .with(SnakeSegment)
            .with(Position { x: 3, y: 3 })
            .with(Size::square(0.8))
            .current_entity()
            .unwrap(),
        spawn_segment(
            commands,
            &materials.segment_material,
            Position { x: 3, y: 2 },
        ),
    ];
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

fn snake_movement(
    keyboard_input: Res<Input<KeyCode>>,
    snake_timer: ResMut<SnakeMoveTimer>,
    segments: ResMut<SnakeSegments>,
    mut last_tail_pos: ResMut<LastTailPosition>,
    mut game_over_events: ResMut<Events<GameOverEvent>>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    if let Some((head_entity, mut head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
        if !snake_timer.0.finished() {
            return;
        }
        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        }
        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_events.send(GameOverEvent);
        }
        if segment_positions.contains(&head_pos) {
            game_over_events.send(GameOverEvent);
        }
        segment_positions.iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
            *positions.get_mut(*segment).unwrap() = *pos;
        });

        last_tail_pos.0 = Some(*segment_positions.last().unwrap());
    }
}

fn resize_window_check(
    windows: Res<Windows>,
    mut window_size: ResMut<WindowSize>
) {
    let window = windows.get_primary().unwrap();
    let current_height = window.height();
    let current_width = window.width();

    if current_height != window_size.height || current_width != window_size.width {
        window_size.height = window.height();
        window_size.width = window.width();
    }
}


fn game_over(
    commands: &mut Commands,
    mut reader: Local<EventReader<GameOverEvent>>,
    game_over_events: Res<Events<GameOverEvent>>,
    materials: Res<SnakeMaterials>,
    asset_server: Res<AssetServer>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    score: Query<Entity, With<Score>>,
    segments: Query<Entity, With<SnakeSegment>>,
    last_tail: Query<Entity, With<LastTailPosition>>
) {
    if reader.iter(&game_over_events).next().is_some() {
        for ent in food.iter().chain(segments.iter()).chain(score.iter()).chain(last_tail.iter()) {
            commands.despawn(ent);
        }
        spawn_snake(commands, asset_server, materials, segments_res);
    }
}

fn snake_growth(
    commands: &mut Commands,
    mut game_over_reader: Local<EventReader<GameOverEvent>>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    last_tail_position: Res<LastTailPosition>,
    growth_events: Res<Events<GrowthEvent>>,
    game_over_events: Res<Events<GameOverEvent>>,
    mut segments: ResMut<SnakeSegments>,
    materials: Res<SnakeMaterials>,
) {
    if growth_reader.iter(&growth_events).next().is_some() && !game_over_reader.iter(&game_over_events).next().is_some() {
        segments.0.push(spawn_segment(
            commands,
            &materials.segment_material,
            last_tail_position.0.unwrap(),
        ));
    }
}

fn snake_timer(time: Res<Time>, mut snake_timer: ResMut<SnakeMoveTimer>) {
    snake_timer.0.tick(time.delta_seconds());
}

#[derive(Default)]
struct LastTailPosition(Option<Position>);
struct GrowthEvent;
#[derive(Default, Debug)]
struct Score(u32);

fn score_board(
    growth_events: Res<Events<GrowthEvent>>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    mut score_query: Query<(&mut Score, &mut Text)>,
){
    if growth_reader.iter(&growth_events).next().is_some() {
        for (mut score, mut text) in score_query.iter_mut(){
            score.0 = score.0 + 1;
            text.value = format!("{}", score.0);
        }
    }
}

fn snake_eating(
    commands: &mut Commands,
    snake_timer: ResMut<SnakeMoveTimer>,
    mut growth_events: ResMut<Events<GrowthEvent>>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    if !snake_timer.0.finished() {
        return;
    }
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.despawn(ent);
                growth_events.send(GrowthEvent);
            }
        }
    }
}

fn start_screen(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
){
    let button_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands
        .spawn(CameraUiBundle::default())
        .spawn(ButtonBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        }).with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    value: "Q-learning".to_string(),
                    font: button_font.clone(),
                    style: TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                },
                ..Default::default()
            }).with(GameMode { value: Games::QLearn});
        })
        .spawn(ButtonBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        }).with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    value: "Deep learning".to_string(),
                    font: button_font.clone(),
                    style: TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                },
                ..Default::default()
            }).with(GameMode { value: Games::DeepLearn});
        })
        .spawn(ButtonBundle {
                style: Style {
                    margin: Rect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                material: button_materials.normal.clone(),
                ..Default::default()
            }).with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text {
                        value: "Human".to_string(),
                        font: button_font.clone(),
                        style: TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                }).with(GameMode { value: Games::Human });
        });
}

fn menu(
    commands: &mut Commands,
    mut state: ResMut<State<AppState>>,
    button_materials: Res<ButtonMaterials>,
    mut game_mode_events: ResMut<Events<GameMode>>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>),
        (Mutated<Interaction>, With<Button>),
    >,
    buttons_query: Query<(Entity, &Button)>
) {
    for (interaction, mut material) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed.clone();
                state.set_next(AppState::InGame).unwrap();
                for (entity, _) in buttons_query.iter(){
                    commands.despawn_recursive(entity);
                }
                game_mode_events.send(GameMode { value: Games::DeepLearn})
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}

const STAGE: &str = "game";
#[derive(Clone)]
enum AppState {
    Menu,
    InGame,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_resource(WindowDescriptor {
            title: "Snake".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .add_resource(SnakeMoveTimer(Timer::new(
            Duration::from_millis(250. as u64),
            true,
        )))
        .add_resource(WindowSize {
            height: 500.0,
            width: 500.0
        })
        .init_resource::<ButtonMaterials>()
        .init_resource::<SnakeMaterials>()
        .add_resource(Score::default())
        .add_resource(State::new(AppState::Menu))
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_resource(SnakeSegments::default())
        .add_resource(LastTailPosition::default())
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_event::<GameMode>()
        .add_stage_after(stage::UPDATE, STAGE, StateStage::<AppState>::default())
        .on_state_enter(STAGE, AppState::Menu, start_screen.system())
        .on_state_update(STAGE, AppState::Menu, menu.system())
        .on_state_enter(STAGE, AppState::InGame, setup.system())
        .on_state_enter(STAGE, AppState::InGame, spawn_snake.system())
        .on_state_update(STAGE, AppState::InGame, position_translation.system())
        .on_state_update(STAGE, AppState::InGame, snake_movement.system())
        .on_state_update(STAGE, AppState::InGame, snake_timer.system())
        .on_state_update(STAGE, AppState::InGame, snake_growth.system())
        .on_state_update(STAGE, AppState::InGame, score_board.system())
        .on_state_update(STAGE, AppState::InGame, size_scaling.system())
        .on_state_update(STAGE, AppState::InGame, snake_eating.system())
        .on_state_update(STAGE, AppState::InGame, food_spawner.system())
        .on_state_update(STAGE, AppState::InGame, game_over.system())
        .add_system(resize_window_check.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}