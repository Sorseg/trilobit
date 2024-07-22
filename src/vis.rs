use bevy::{
    color::palettes::{css::{RED, SANDY_BROWN}, tailwind::{RED_100, RED_200, RED_300, RED_400, RED_500}},
    input::mouse::MouseMotion,
    prelude::*,
};
use glam::Vec3 as GV3;

use crate::Hull;

#[derive(Resource, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum HullProgress {
    None,
    InitHull,
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
                transform: Transform::from_xyz(2.0, 3.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
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
                    GV3::new(0.3, 0.4, 0.5),
                    GV3::new(0.6, 0.7, 0.8),
                    GV3::new(0.7, 0.3, 0.2),
                ];

                let mut hull = Hull::new(pts);
                if *state as u8 >= HullProgress::InitHull as u8 {
                    hull.populate_initial_hull();
                }

                for (i,h) in hull.faces.iter().copied().enumerate() {
                    let c = Color::srgb(i as f32 * 10.0, 0.0, 0.5);
                    for p in h {
                        gizmos.sphere(hull.points[p].to_bevy(), Quat::IDENTITY, 0.03, c);
                    }
                    for [fr,t] in [[0, 1], [1, 2], [2, 0]] {
                        gizmos.line(
                            hull.points[h[fr]].to_bevy(),
                            hull.points[h[t]].to_bevy(),
                            c
                        )
                    }
                }
                for &GV3 { x, y, z } in hull.points {
                    gizmos.sphere(Vec3 { x, y, z }, Quat::IDENTITY, 0.03, SANDY_BROWN);
                }
                if input.just_pressed(KeyCode::Space) {
                    match *state {
                        HullProgress::None => {
                            *state = HullProgress::InitHull;
                            println!("Initializing hull")
                        }
                        HullProgress::InitHull => todo!(),
                    }
                }
            },
        )
        .run();
}

fn rotate_cam(mut cam: Query<&mut Transform, With<Camera>>, mut input: EventReader<MouseMotion>) {
    let sens = 0.008;
    let delta = input.read().map(|e| e.delta).sum::<Vec2>() * sens;
    let rot =  * Quat::from_rotation_y(-delta.y);

    let mut cam = cam.single_mut();
    cam.rotate_around(Vec3::ZERO, Quat::from_rotation_y(-delta.x));
    
}
