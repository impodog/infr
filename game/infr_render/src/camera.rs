use bevy::{
    camera::{CameraOutputMode, ImageRenderTarget, RenderTarget, visibility::RenderLayers},
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use infr_client::config;

/// The resolution that the pixel camera renders to.
#[derive(Resource, Debug, Clone)]
pub struct VirtualResolution {
    pub x: u32,
    pub y: u32,
}
impl Default for VirtualResolution {
    fn default() -> Self {
        VirtualResolution { x: 400, y: 225 }
    }
}

pub const PIXEL_RENDER_LAYER: RenderLayers = RenderLayers::layer(0);
pub const DISPLAY_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

#[derive(Component)]
pub struct PixelCamera;

#[derive(Component)]
pub struct MainCamera;

pub(crate) fn setup_camera(
    mut commands: Commands,
    resolution: Res<VirtualResolution>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: resolution.x,
        height: resolution.y,
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
    let texture = images.add(texture);

    // The camera to output to a texture, which creates a pixel effect.
    commands.spawn((
        PixelCamera,
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
        Sprite {
            image: texture,
            custom_size: Some(Vec2::new(
                config::STARTUP_CONFIG.window_size.0 as f32,
                config::STARTUP_CONFIG.window_size.1 as f32,
            )),
            ..Default::default()
        },
        DISPLAY_RENDER_LAYER,
    ));
}
