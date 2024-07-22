use bevy::{
    color::palettes::css::{ORANGE, SANDY_BROWN},
    input::mouse::MouseMotion,
    prelude::*,
};
use glam::Vec3 as GV3;
use rand::{RngCore, SeedableRng};

use crate::Hull;

#[derive(Resource, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum HullProgress {
    None,
    InitHull,
    FillConflicts,
}

trait GToB {
    fn to_bevy(self) -> Vec3;
}

impl GToB for GV3 {
    fn to_bevy(self) -> Vec3 {
        let GV3 { x, y, z } = self;
        Vec3 { x, y, z }
    }
}

pub fn vis() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(HullProgress::None)
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera3dBundle {
                transform: Transform::from_xyz(2.0, 3.0, 2.5).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            });
        })
        .add_systems(Update, rotate_cam)
        .add_systems(
            Update,
            |mut gizmos: Gizmos,
             mut state: ResMut<HullProgress>,
             input: Res<ButtonInput<KeyCode>>| {
                let pts = &[
                    // edges of the cube
                    GV3::new(0.0, 0.0, 0.0),
                    GV3::new(0.0, 0.0, 1.0),
                    GV3::new(0.0, 1.0, 0.0),
                    GV3::new(0.0, 1.0, 1.0),
                    GV3::new(1.0, 0.0, 0.0),
                    GV3::new(1.0, 0.0, 1.0),
                    GV3::new(1.0, 1.0, 0.0),
                    GV3::new(1.0, 1.0, 1.0),
                    // some internal noise
                    // GV3::new(0.3, 0.4, 0.5),
                    // GV3::new(0.6, 0.7, 0.8),
                    // GV3::new(0.7, 0.3, 0.2),
                ];

                let mut hull = Hull::new(pts);
                if *state >= HullProgress::InitHull {
                    hull.populate_initial_hull();
                }
                if *state >= HullProgress::FillConflicts {
                    hull.fill_conflicts();
                }
                let mut rand = rand::rngs::StdRng::seed_from_u64(0);

                for (i, h) in hull.faces.iter().enumerate() {
                    let offset = Vec3::from_array(std::array::from_fn(|_| {
                        rand.next_u32() as f32 / u32::MAX as f32 * 0.1
                    }));
                    let center = h
                        .indices
                        .map(|i| hull.points[i].to_bevy())
                        .into_iter()
                        .sum::<Vec3>()
                        / 3.0;
                    let c = Color::srgb(i as f32 / 5.0, 0.5, 0.0);
                    for p in h.indices {
                        gizmos.sphere(hull.points[p].to_bevy(), Quat::IDENTITY, 0.03, c);
                    }
                    for [fr, t] in [[0, 1], [1, 2], [2, 0]] {
                        gizmos
                            .arrow(
                                hull.points[h.indices[fr]].to_bevy() + offset,
                                hull.points[h.indices[t]].to_bevy() + offset,
                                c,
                            )
                            .with_tip_length(0.05);
                    }
                    for p in &h.conflicts {
                        gizmos.arrow(center, hull.points[*p].to_bevy(), ORANGE).with_tip_length(0.1);
                    }
                }
                for &GV3 { x, y, z } in hull.points {
                    gizmos.sphere(Vec3 { x, y, z }, Quat::IDENTITY, 0.02, SANDY_BROWN);
                }
                if input.just_pressed(KeyCode::Space) {
                    *state = match *state {
                        HullProgress::None => HullProgress::InitHull,
                        HullProgress::InitHull => HullProgress::FillConflicts,
                        HullProgress::FillConflicts => todo!(),
                    };
                    println!("{:?}", *state);
                }
            },
        )
        .run();
}

fn rotate_cam(mut cam: Query<&mut Transform, With<Camera>>, mut input: EventReader<MouseMotion>) {
    let sens = 0.008;
    let delta = input.read().map(|e| e.delta).sum::<Vec2>() * sens;

    let mut cam = cam.single_mut();
    cam.rotate_around(Vec3::ZERO, Quat::from_rotation_y(-delta.x));
    cam.rotate_around(Vec3::ZERO, Quat::from_rotation_x(-delta.y));
}
