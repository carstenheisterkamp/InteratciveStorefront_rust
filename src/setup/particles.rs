use ::bevy_hanabi::prelude::*;
use ::bevy_hanabi::{Gradient, ParticleTextureModifier, ImageSampleMapping, EffectMaterial};
use bevy::prelude::*;
use crate::setup::assetloader::LoadedTextures;

#[derive(Resource)]
pub struct MyEffectHandle(pub Handle<EffectAsset>);

pub fn setup(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    // Writer für Ausdrücke (inkl. AGE, Random, Rechen-OPS)
    let w = ExprWriter::new();

    let lifetime_secs: f32 = 15.0;
    let lifetime = w.lit(lifetime_secs).expr();
    let fade_in_secs: f32 = 1.5;
    let fade_in_t: f32 = fade_in_secs / lifetime_secs;
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1.0, 1.0, 1.0, 0.0));        // transparent am Start
    gradient.add_key(fade_in_t, Vec4::new(1.0, 1.0, 1.0, 1.0)); // vollständig sichtbar nach fade_in_t
    gradient.add_key(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0));        // am Ende ausfaden

    // Init-/Update-Parameter
    let center = w.lit(Vec3::ZERO).expr();
    let radius = w.lit(10.5).expr();
    let speed = w.lit(0.1).expr();
    let accel = w.lit(Vec3::new(0.0, 0.0, 0.0)).expr();

    // Billboard-Rotation: zufällige langsame Winkelgeschwindigkeit pro Partikel
    // omega in [-0.6, 0.6] rad/s (langsam) => (rand*2-1) * 0.6
    let rand01 = w.rand(ScalarType::Float);
    let omega = (rand01 * w.lit(2.0) - w.lit(1.0)) * w.lit(0.6);
    // Drehwinkel = AGE * omega
    let age = w.attr(Attribute::AGE);
    let rotation = (age * omega).expr();

    let min_size: f32 = 0.001;
    let max_size: f32 = 0.005;

    let rand02 = w.rand(ScalarType::Float);
    let size = (rand02 * w.lit(100) * w.lit(max_size - min_size) + w.lit(min_size)).expr();

    // Module finalisieren, dann Texture-Slot anlegen
    let mut module = w.finish();
    module.add_texture_slot("color");
    let texture_slot = module.lit(0u32);

    // Modifiers
    let init_pos = SetPositionSphereModifier {
        center,
        radius,
        dimension: ShapeDimension::Volume,
    };

    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed,
    };

    let init_scale = SetAttributeModifier::new(Attribute::SIZE, size);

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let update_accel = AccelModifier::new(accel);

    let effect = EffectAsset::new(
        32768,
        SpawnerSettings::rate(5.0.into()),
        module
    )
        .with_name("MyEffect")
        .init(init_pos)
        .init(init_scale)
        .init(init_vel)
        .init(init_lifetime)
        .update(update_accel)
        // Kamera-facing (Billboard) + in-plane Rotation per Partikel
        .render(OrientModifier::new(OrientMode::ParallelCameraDepthPlane).with_rotation(rotation))
        .render(ColorOverLifetimeModifier { gradient, ..default() })
        .render(ParticleTextureModifier {
            texture_slot,
            sample_mapping: ImageSampleMapping::Modulate,
        });

    let effect_handle = effects.add(effect);
    commands.insert_resource(MyEffectHandle(effect_handle));
}

pub fn spawn_particlesystem(
    mut commands: Commands,
    effect_handle: Res<MyEffectHandle>,
    loaded_textures: Res<LoadedTextures>,
) {
    // Binde die Textur über EffectMaterial an Slot 0
    let images = loaded_textures
        .dust_particle
        .as_ref()
        .map(|h| vec![h.clone()])
        .unwrap_or_default();

    commands.spawn((
        ParticleEffect::new(effect_handle.0.clone()),
        EffectMaterial { images },
        Transform::from_translation(Vec3::ZERO)
    ));
}