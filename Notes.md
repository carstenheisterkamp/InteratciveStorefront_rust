# Notes

## Links
https://bevy-cheatbook.github.io/
https://github.com/BlackPhlox/bevy_osc
https://crates.io/crates/noise
https://github.com/glslify/glsl-easings

https://opengameart.org/

ch habe die Hanabi-API an der von dir verlinkten Stelle geprüft und den Code darauf abgestimmt. Kurz: In bevy_hanabi 0.17 funktioniert die Partikel-Textur so:
Definiere Texture-Slots im Module mit module.add_texture_slot("…").
Referenziere den Slot im Shader mit einem ExprHandle (hier: module.lit(0u32)).
Füge den ParticleTextureModifier hinzu; er hat genau zwei Felder:
texture_slot: ExprHandle
sample_mapping: ImageSampleMapping (z. B. Modulate, ModulateRGB, ModulateOpacityFromR)
Binde die eigentlichen Texturen pro Instanz über die Komponente EffectMaterial { images } in der gleichen Entity wie ParticleEffect.
Was ich umgesetzt habe
In setup():
module.add_texture_slot("color") definiert den Slot.
texture_slot = module.lit(0u32) erzeugt den Ausdruck für Slot 0.
Der Effekt bekommt .render(ParticleTextureModifier { texture_slot, sample_mapping: ImageSampleMapping::Modulate }).
In spawn_particlesystem():
Ich hole die Textur aus LoadedTextures.smoke_particle und übergebe sie als erstes (und einziges) Element in EffectMaterial.images, sodass sie in Slot 0 landet.
Der ParticleEffect und das EffectMaterial liegen auf derselben Entity.
Warum die früheren Fehler auftraten
with_property / add_property gibt es an EffectAsset nicht mehr (auf Module-Ebene wurde add_texture_slot eingeführt).
with_image(image) gibt es in 0.17 nicht an EffectAsset.
ParticleTextureModifier hat keine Felder wie image/texture; korrekt sind texture_slot und sample_mapping. Es gibt auch keinen ::default()-Konstruktor mit Bild-Handle.
Genau das bestätigt auch die 0.17-Quelle: ParticleTextureModifier { texture_slot: ExprHandle, sample_mapping: ImageSampleMapping } mit new(texture_slot).
Build-Check
Ich habe einen cargo check ausgeführt: Keine Fehler, nur ein paar Warnungen in anderen Dateien. Partikelcode baut sauber.
Hinweise
Achte darauf, dass in assets/config/settings.json ein Eintrag "smoke_particle" existiert, sonst bleibt images leer. Mit ImageSampleMapping::Modulate ergibt das effektiv “schwarze” Samples; Partikel können unsichtbar wirken. Bei leerem Handle könntest du optional eine Warnung loggen.
Wenn du Alpha über die Textur steuern willst, setze sample_mapping auf ImageSampleMapping::ModulateOpacityFromR (klassisch für Masken in R).
Damit ist die Verkabelung “Assetloader -> Hanabi-Slot -> Material -> Renderer” korrekt gemäß bevy_hanabi 0.17 und die Textur aus dem Assetloa