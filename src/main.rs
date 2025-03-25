use bevy::{
    prelude::*,
    reflect::TypePath, 
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        mesh::MeshVertexBufferLayoutRef,
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::{texture_2d_array, texture_storage_2d}, *},
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin}
};
use std::borrow::Cow;

const SHADER_ASSET_PATH: &str = "shaders/custom_gltf_2d.wgsl";

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Rust Painter".to_string(),
                    resolution: (512.0, 512.0).into(),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            PainterPlugin,
            Material2dPlugin::<CustomMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let pattern_0 = asset_server.load("textures/pattern_0.png");
    


    let mut mixed_image = Image::new_fill(
        Extent3d { width: 512, height: 512, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0, 1, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    mixed_image.texture_descriptor.usage = TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let mixed_image_handle = images.add(mixed_image);
    
    // Add a 512 squre mesh
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(materials.add(CustomMaterial {uv_texture: mixed_image_handle.clone()})),
        Transform::from_scale(512.0 * Vec3::ONE),
    ));
    
    // Add a camera
    commands.spawn(Camera2d);

    commands.insert_resource(MixedPattern {
        loaded: false,
        pattern_0: pattern_0,
        mixed_pattern: mixed_image_handle
    });
}

fn update(
    mut pattern: ResMut<MixedPattern>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    if pattern.loaded         
    || !asset_server
    .load_state(pattern.pattern_0.id())
    .is_loaded() {
        return;
    }

    pattern.loaded = true;
    let pattern_image = images.get_mut(&pattern.pattern_0).unwrap();
    pattern_image.reinterpret_stacked_2d_as_array(2);
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub uv_texture : Handle<Image>
}

impl Material2d for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            // Mesh::ATTRIBUTE_COLOR.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            // ATTRIBUTE_BARYCENTRIC.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

#[derive(Resource, Clone, ExtractResource)]
struct MixedPattern {
    loaded: bool,
    pattern_0: Handle<Image>,
    mixed_pattern: Handle<Image>,
}

struct PainterPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PainterLabel;

impl Plugin for PainterPlugin {
    fn build(&self, app: &mut App) {
        // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        app.add_plugins(ExtractResourcePlugin::<MixedPattern>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
        );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(PainterLabel, MixerNode::default());
        render_graph.add_node_edge(PainterLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<MixerPipeline>();
    }
}

// #[derive(Resource, Clone, ExtractResource)]
// struct PatternLayers {
//     pattern_0 : Handle<Image>,
// }

#[derive(Resource)]
struct PainterBindGroups([BindGroup; 1]);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<MixerPipeline>,
    // pattern_layers : Res<PatternLayers>,
    mixed_pattern : Res<MixedPattern>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
) {
    if !mixed_pattern.loaded {
        return;
    }
    
    let pattern_gpu_image = gpu_images.get(&mixed_pattern.pattern_0).unwrap();
    
    let mixed_gpu_image = gpu_images.get(&mixed_pattern.mixed_pattern).unwrap();
    
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &BindGroupEntries::sequential((&pattern_gpu_image.texture_view, &mixed_gpu_image.texture_view)),
    );

    commands.insert_resource(PainterBindGroups([bind_group]));
}

#[derive(Resource)]
struct MixerPipeline {
    texture_bind_group_layout: BindGroupLayout,
    main_pipeline: CachedComputePipelineId,
}

impl FromWorld for MixerPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let texture_bind_group_layout = render_device.create_bind_group_layout(
            "MixedPattern",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    texture_2d_array(TextureSampleType::Float { filterable: true }),
                    texture_storage_2d(TextureFormat::Rgba8Unorm, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        let shader = world.load_asset("shaders/mixer.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let main_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
            zero_initialize_workgroup_memory: false,
        });

        MixerPipeline{
            texture_bind_group_layout,
            main_pipeline,
        }
    }    
}

struct MixerNode {}

impl Default for MixerNode {
    fn default() -> Self {
        Self {}
    }
}

impl render_graph::Node for MixerNode {
    // fn update(&mut self, world: &mut World) {
    //     let pipeline = world.resource::<MixerPipeline>();
    //     let pipeline_cache = world.resource::<PipelineCache>();
    // }
    fn run<'w>(
            &self,
            _graph: &mut render_graph::RenderGraphContext,
            render_context: &mut RenderContext<'w>,
            world: &'w World,
        ) -> Result<(), render_graph::NodeRunError> {
            if !world.resource::<MixedPattern>().loaded {
                return Ok(());
            }

            let bind_groups = &world.resource::<PainterBindGroups>().0;
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<MixerPipeline>();        

            let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

            let main_pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.main_pipeline)
            .unwrap();
            pass.set_bind_group(0, &bind_groups[0], &[]);
            pass.set_pipeline(main_pipeline);
            pass.dispatch_workgroups(512 / 8, 512 / 8, 1);

            Ok(())
    }
}