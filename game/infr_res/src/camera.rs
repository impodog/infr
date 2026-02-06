use bevy::{
    camera::{CameraOutputMode, ImageRenderTarget, RenderTarget, visibility::RenderLayers},
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use infr_client::config;

/// The resolution that the pixel camera renders to.
#[derive(Resource, Debug, Clone, Copy)]
pub struct VirtualResolution {
    pub x: u32,
    pub y: u32,
}
impl Default for VirtualResolution {
    fn default() -> Self {
        VirtualResolution {
            x: config::CONFIG.display.window_size.0,
            y: config::CONFIG.display.window_size.1,
        }
    }
}

pub const PIXEL_RENDER_LAYER: RenderLayers = RenderLayers::layer(0);
pub const DISPLAY_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

#[derive(Component)]
pub struct PixelCamera(pub Handle<Image>);

#[derive(Component)]
pub struct MainCamera;

/// Marks the sprite to render to.
#[derive(Component)]
#[require(Sprite)]
pub struct PixelRenderSprite;

fn make_window_texture(images: &mut Assets<Image>) -> Handle<Image> {
    // Sets the maximum size for rendering pixel graphics.
    let size = Extent3d {
        width: config::CONFIG.display.window_size.0,
        height: config::CONFIG.display.window_size.1,
        depth_or_array_layers: 1,
    };
    // Create the render texture
    let mut texture = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    // Fill image.data with zeroes
    texture.resize(size);
    images.add(texture)
}

pub(crate) fn setup_camera(
    mut commands: Commands,
    resolution: Res<VirtualResolution>,
    mut images: ResMut<Assets<Image>>,
) {
    let texture = make_window_texture(&mut images);
    // The camera to output to a texture, which creates a pixel effect.
    commands.spawn((
        PixelCamera(texture.clone()),
        Camera {
            output_mode: CameraOutputMode::Write {
                blend_state: None,
                clear_color: ClearColorConfig::Custom(Color::NONE),
            },
            order: -1,
            ..Default::default()
        },
        RenderTarget::Image(ImageRenderTarget {
            handle: texture.clone(),
            scale_factor: 1.0,
        }),
        Msaa::Off,
        Camera2d,
        PIXEL_RENDER_LAYER,
    ));

    // The camera that renders to the display.
    commands.spawn((
        MainCamera,
        Camera2d,
        IsDefaultUiCamera,
        Msaa::Off,
        DISPLAY_RENDER_LAYER,
    ));

    commands.spawn((
        PixelRenderSprite,
        Sprite {
            image: texture,
            custom_size: Some(Vec2::new(
                config::CONFIG.display.window_size.0 as f32,
                config::CONFIG.display.window_size.1 as f32,
            )),
            rect: Some(Rect::new(
                0.0,
                0.0,
                resolution.x as f32,
                resolution.y as f32,
            )),
            ..Default::default()
        },
        DISPLAY_RENDER_LAYER,
    ));
}

pub(crate) fn update_camera(
    resolution: Res<VirtualResolution>,
    mut sprite: Query<&mut Sprite, With<PixelRenderSprite>>,
) -> Result<()> {
    if resolution.is_changed() {
        let mut sprite = sprite.single_mut()?;
        sprite.rect = Some(Rect::from_center_size(
            Vec2::new(
                config::CONFIG.display.window_size.0 as f32 * 0.5,
                config::CONFIG.display.window_size.1 as f32 * 0.5,
            ),
            Vec2::new(resolution.x as f32, resolution.y as f32),
        ));
    }
    Ok(())
}

pub(crate) fn update_resolution(
    mut resolution: ResMut<VirtualResolution>,
    center: Res<infr_client::PositionCenter>,
    mut prev_center: Local<infr_client::PositionCenter>,
) {
    if *center != *prev_center {
        *prev_center = *center;
        let x = config::CONFIG.display.tile_size.0 as f32 * center.extent.x * 2.0;
        let y = config::CONFIG.display.tile_size.1 as f32 * center.extent.y * 2.0;
        let ratio = config::CONFIG.display.window_size.0 as f32
            / config::CONFIG.display.window_size.1 as f32;
        let (x, y) = if x / y > ratio {
            (x, x / ratio)
        } else {
            (y * ratio, y)
        };
        resolution.x = x as u32;
        resolution.y = y as u32;
    }
}

pub(crate) fn update_color_tinting(
    meta: Res<infr_client::LevelMetadata>,
    mut q_sprite: Query<&mut Sprite, With<PixelRenderSprite>>,
) -> Result<()> {
    if meta.is_changed()
        && let Some(ref tinting) = meta.tinting
    {
        let mut sprite = q_sprite.single_mut()?;
        if let Some(color) = config::CONFIG.sprites.tinting.get(tinting) {
            info!("Set color tint: {color:?}");
            sprite.color = *color;
        } else {
            error!("Unknown tinting scheme: {tinting}");
            sprite.color = Default::default();
        }
    }
    Ok(())
}
