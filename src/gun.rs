use crate::collision::ColliderShape;
use crate::prelude::*;
use crate::quadtree::quad_collider::Shape;
use crate::{
    components::Damage,
    player::Player,
    resources::{CursorPos, GlobTextAtlases},
};

use bevy::math::vec2;
use bevy::utils::Instant;
use bevy::{prelude::*, time::Stopwatch};
use std::f32::consts::PI;

pub struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameInit), spawn_gun)
            .add_systems(
                Update,
                (handle_gun_input, update_gun_pos, update_bullet_pos)
                    .run_if(in_state(GameState::GameRun)),
            )
            .add_systems(Last, despawn_bullets.run_if(in_state(GameState::GameRun)));
    }
}

#[derive(Component)]
#[require(Transform, Sprite, GunTimer)]
pub struct Gun;

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct GunTimer(pub Stopwatch);

#[derive(Component)]
#[require(
    Transform,
    Sprite,
    BulletDirection,
    Damage,
    SpawnInstant(|| SpawnInstant(Instant::now())),
    ColliderShape(|| ColliderShape(Shape::Circle(Circle::new(4.0))))
)]
pub struct Bullet;

#[derive(Component, Debug, Deref, DerefMut)]
pub struct SpawnInstant(pub Instant);

#[derive(Component, Debug, Deref, DerefMut, Default)]
pub struct BulletDirection(Vec2);

fn spawn_gun(mut commands: Commands, text_atlases: Res<GlobTextAtlases>) {
    let layout = text_atlases.common.clone().unwrap().layout;
    let image = text_atlases.common.clone().unwrap().image;

    // Gun
    commands.spawn((
        Sprite::from_atlas_image(image, TextureAtlas { layout, index: 10 }),
        Transform::from_translation(Vec3::new(0., 0., 55.)),
        GunTimer(Stopwatch::new()),
        Gun,
    ));
}

fn handle_gun_input(
    mut cmds: Commands,
    mut gun_query: Query<(&mut GunTimer, &Transform), With<Gun>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    text_atlases: Res<GlobTextAtlases>,
    time: Res<Time>,
) {
    let (mut gun_timer, gun_transf) = gun_query.single_mut();
    gun_timer.tick(time.delta());

    if mouse_input.pressed(MouseButton::Left)
        && gun_timer.elapsed_secs() >= BULLET_SPAWN_INTERVAL_SECS
    {
        let gun_pos = gun_transf.translation.truncate();
        let bullet_dir = gun_transf.local_x().truncate().normalize_or_zero();
        let layout = text_atlases.common.clone().unwrap().layout;
        let image = text_atlases.common.clone().unwrap().image;

        gun_timer.reset();
        cmds.spawn((
            Sprite::from_atlas_image(image, TextureAtlas { layout, index: 11 }),
            // Spawn between the player and the gun on Z-axis
            Transform::from_translation(gun_pos.extend(52.5)).with_scale(Vec3::splat(0.95)),
            Bullet,
            BulletDirection(bullet_dir),
            Damage(10),
        ));
    }
}

fn update_gun_pos(
    mut gun_query: Query<&mut Transform, (With<Gun>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    cursor_pos: Res<CursorPos>,
) {
    let player_pos = player_query.single().translation.truncate();
    let mut gun_transf = gun_query.single_mut();
    let cursor_pos = cursor_pos.unwrap_or(player_pos);

    let angle = (player_pos.y - cursor_pos.y).atan2(player_pos.x - cursor_pos.x) + PI;
    gun_transf.rotation = Quat::from_rotation_z(angle);

    let offs = 4.;
    let new_gun_pos = vec2(
        player_pos.x + offs * angle.cos(),
        player_pos.y + offs * angle.sin() - 4.,
    );

    gun_transf.translation = new_gun_pos.extend(gun_transf.translation.z);
}

fn update_bullet_pos(
    mut bullet_query: Query<(&mut Transform, &BulletDirection), With<Bullet>>,
    time: Res<Time>,
) {
    if bullet_query.is_empty() {
        return;
    }

    bullet_query.iter_mut().for_each(|(mut t, dir)| {
        t.translation += (**dir * BULLET_SPEED * time.delta_secs()).extend(0.);
    });
}

fn despawn_bullets(
    mut commands: Commands,
    bullet_query: Query<(Entity, &SpawnInstant), With<Bullet>>,
) {
    bullet_query.iter().for_each(|(ent, inst)| {
        if inst.elapsed().as_secs_f32() >= BULLET_LIFE_SECS {
            commands.entity(ent).despawn()
        }
    });
}
