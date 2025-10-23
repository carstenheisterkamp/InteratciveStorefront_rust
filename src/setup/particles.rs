use ::bevy_hanabi::prelude::*;
use::bevy_hanabi::Gradient;
use bevy::prelude::*;

#[derive(Resource)]
pub struct MyEffectHandle(pub Handle<EffectAsset>);

pub fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1., 0., 0., 1.));
    gradient.add_key(1.0, Vec4::splat(0.));

    let mut module = Module::default();

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(2.),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(6.),
    };

    let lifetime = module.lit(10.);
    let init_lifetime = SetAttributeModifier::new(
        Attribute::LIFETIME, lifetime);

    // Every frame, add a gravity-like acceleration downward
    let accel = module.lit(Vec3::new(0., -3., 0.));
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset
    let effect = EffectAsset::new(
        32768,
        SpawnerSettings::rate(5.0.into()),
        module
    )
        .with_name("MyEffect")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .update(update_accel)
        .render(ColorOverLifetimeModifier { gradient, ..default() });

    // Insert into the asset system
    let effect_handle = effects.add(effect);
    commands.insert_resource(MyEffectHandle(effect_handle));
}

pub fn spawn_particlesystem(
    mut commands: Commands,
    effect_handle: Res<MyEffectHandle>
) {
    commands.spawn((
        ParticleEffect::new(effect_handle.0.clone()),
        Transform::from_translation(Vec3::Y),
    ));
}