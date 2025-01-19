#![allow(dead_code)]

use std::f32::consts::PI;

use bevy::{prelude::*, render::mesh::morph::MeshMorphWeights};
use facial_anim::FacialAnim;

mod com;
mod facial_anim;

fn main() -> AppExit {
    // let ctx = context::initialize(CHARACTER_DATA.to_vec(), ALGORITHM_DATA.to_vec()).unwrap();
    // let player = ctx
    //     .add_player(SG_SampleType::SG_SAMPLE_PCM16, SG_SampleRate::SG_RATE_16KHZ)
    //     .unwrap();
    // let samples_10ms = player.sample_rate().to_rate() / (1000 / 10); // Samples per 10ms
    // let mut buffer: Vec<u16> = vec![0; samples_10ms as usize];

    // loop {
    //     buffer.fill(0);
    //     player.add_input_pcm16(&mut buffer).unwrap();
    //     dbg!(&player);
    //     let output = player.process(Duration::from_millis(10)).unwrap();
    //     dbg!(output);
    //     thread::sleep(Duration::from_millis(10));
    // }

    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
            ..Default::default()
        }))
        .add_plugins(facial_anim::FacialAnimPlugin)
        .insert_resource(AmbientLight {
            brightness: 100.,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (name_morphs, setup_animations, run_anim))
        .run()
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.init_resource::<MorphNames>();
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("miku.glb")),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 500.0,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_rotation_z(PI / 2.0)),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 0.5).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));
}

fn setup_animations(
    mut has_setup: Local<bool>,
    mut commands: Commands,
    mut players: Query<(Entity, &Name, &Mesh3d, &Parent)>,
    meshes: Res<Assets<Mesh>>,
) {
    if *has_setup {
        return;
    }

    for (entity, name, mesh3d, parent) in &mut players {
        if let Some(mesh) = meshes.get(mesh3d) {
            if !mesh.has_morph_targets() {
                continue;
            }

            info!("Name: {name}");
            let weights = vec![0.0; mesh.morph_target_names().unwrap().len()];
            commands.entity(parent.get()).insert_if_new(
                MorphWeights::new(weights.clone(), Some(mesh3d.0.clone_weak())).unwrap(),
            );

            commands
                .entity(entity)
                .insert(MeshMorphWeights::new(weights).unwrap());

            *has_setup = true;
        }
    }
}

#[derive(Resource, Default)]
pub struct MorphNames(Vec<String>);

fn name_morphs(
    mut has_printed: Local<bool>,
    morph_data: Query<&MorphWeights>,
    mut morph_names: ResMut<MorphNames>,
    meshes: Res<Assets<Mesh>>,
) {
    if *has_printed {
        return;
    }

    let Some(morph_data) = morph_data.iter().next() else {
        return;
    };

    let Some(mesh) = morph_data.first_mesh() else {
        return;
    };

    let Some(mesh) = meshes.get(mesh) else {
        return;
    };
    let Some(names) = mesh.morph_target_names() else {
        return;
    };

    info!("Target names:");
    for name in names {
        info!("  {name}");
    }
    morph_names.0 = names.to_vec();
    *has_printed = true;
}

fn run_anim(
    mut morph_data: Query<&mut MorphWeights>,
    anim: Res<FacialAnim>,
    names: Res<MorphNames>,
    mut morph_indices: Local<Option<Vec<Option<(usize, usize)>>>>,
) {
    let Some(mut morph_data) = morph_data.iter_mut().next() else {
        return;
    };

    let Some(processed_data) = &anim.processed_data else {
        return;
    };

    let weights = morph_data.weights_mut();

    if morph_indices.is_none() {
        let name_paths = names.0.iter().map(|name| ("blendBoard", name));
        *morph_indices = Some(
            name_paths
                .map(|(node_name, channel_name)| {
                    (
                        (node_name, channel_name),
                        anim.names
                            .iter()
                            .enumerate()
                            .find(|(_, (anim_node_name, _))| anim_node_name == node_name)
                            .map(|(node_index, (_, node_channels))| {
                                (
                                    node_index,
                                    node_channels.iter().position(|anim_channel_name| {
                                        format!("{}_pose", anim_channel_name) == *channel_name
                                    }),
                                )
                            }),
                    )
                })
                .map(|((node_name, channel_name), result)| {
                    if let Some((node_index, Some(channel_index))) = result {
                        Some((node_index, channel_index))
                    } else {
                        warn!("Could not find morph target: {node_name} {channel_name}");
                        None
                    }
                })
                .collect(),
        );
    }

    for (morph_index, (node_index, channel_index)) in morph_indices
        .as_ref()
        .unwrap()
        .iter()
        .enumerate()
        .filter_map(|(i, x)| x.map(|y| (i, y)))
    {
        weights[morph_index] = processed_data[node_index][channel_index];
    }
    info!("Written weights");
}
