use std::fs::File;
use std::io::Write;

use bevy::{asset::HandleId, prelude::*, reflect::TypeUuid, tasks::IoTaskPool};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Stuff;

#[derive(Component, Reflect, Default)]
#[reflect(Component)] // this tells the reflect derive to also reflect component behaviors
struct SaveMe;

const SCENE_FILE_PATH: &str = "scenes/load_scene_example.scn.ron";

#[derive(Resource)]
struct MyAssets {
    cube_mesh: Handle<Mesh>,
    cube_mat: Handle<StandardMaterial>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..default()
        }))
        //Load some assets
        .add_startup_system_to_stage(StartupStage::PreStartup, add_assets)
        //Register some components beyond the defaults to be extracted during save
        .register_type::<Stuff>()
        .register_type::<SaveMe>()
        .add_startup_system(setup)
        .add_system(spawn_stuff)
        .add_system(save_scene_system)
        .add_system(load_scene_system)
        .add_system(move_stuff)
        .add_system(clear_stuff)
        .add_plugin(WorldInspectorPlugin)
        .run();
}

fn add_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let my_assets = MyAssets {
        cube_mesh: meshes.set(
            HandleId::new(Mesh::TYPE_UUID, 26467347373),
            Mesh::from(shape::Cube { size: 1.0 }),
        ),
        cube_mat: materials.set(
            HandleId::new(StandardMaterial::TYPE_UUID, 151251351533),
            Color::rgb(0.8, 0.7, 0.6).into(),
        ),
    };

    commands.insert_resource(my_assets)
}

fn spawn_stuff(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    my_assets: Res<MyAssets>,
) {
    if !keyboard_input.just_pressed(KeyCode::A) {
        return;
    }
    // cube
    commands
        .spawn(PbrBundle {
            mesh: my_assets.cube_mesh.clone(),
            material: my_assets.cube_mat.clone(),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(Stuff)
        .insert(SaveMe);
}
/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn move_stuff(mut stuff: Query<&mut Transform, With<Stuff>>, keyboard_input: Res<Input<KeyCode>>) {
    for mut transform in stuff.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Up) {
            transform.translation += Vec3::new(0.1, 0.0, 0.0)
        }
        if keyboard_input.just_pressed(KeyCode::Right) {
            transform.translation += Vec3::new(0.0, 0.1, 0.0)
        }
        if keyboard_input.just_pressed(KeyCode::Left) {
            transform.translation += Vec3::new(0.0, -0.1, 0.0)
        }
    }
}

fn clear_stuff(
    stuff: Query<Entity, With<Stuff>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::D) {
        for e in stuff.iter() {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn load_scene_system(
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,

    mut scene_spawner: ResMut<SceneSpawner>,
) {
    if !keyboard_input.just_pressed(KeyCode::L) {
        return;
    }

    scene_spawner.spawn_dynamic(asset_server.load(SCENE_FILE_PATH));
}

fn save_scene_system(
    world: &World,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, With<SaveMe>>,
) {
    if !keyboard_input.just_pressed(KeyCode::S) {
        return;
    }

    // The TypeRegistry resource contains information about all registered types (including
    // components). This is used to construct scenes.
    let type_registry = world.resource::<AppTypeRegistry>();
    let mut scene = DynamicSceneBuilder::from_world(world);
    //Only extract entities which has the SaveMe marker component
    scene.extract_entities(query.iter());
    // Scenes can be serialized like this:
    let serialized_scene = scene.build().serialize_ron(type_registry).unwrap();

    // Showing the scene in the console
    info!("{}", serialized_scene);

    // Writing the scene to a new file. Using a task to avoid calling the filesystem APIs in a system
    // as they are blocking
    // This can't work in WASM as there is no filesystem access
    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create(format!("assets/{SCENE_FILE_PATH}"))
                .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                .expect("Error while writing scene to file");
        })
        .detach();
}
