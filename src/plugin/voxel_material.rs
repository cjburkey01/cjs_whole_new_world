use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexFormat,
        },
        texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    },
};

pub struct VoxelMaterialPlugin;

impl Plugin for VoxelMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VoxelExtendedMaterial>::default())
            .add_systems(Startup, add_chunk_material_system);
    }
}

pub type VoxelExtendedMaterial = ExtendedMaterial<StandardMaterial, VoxelChunkMaterial>;

#[derive(Resource)]
pub struct ChunkMaterialRes(pub Handle<VoxelExtendedMaterial>);

fn add_chunk_material_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<VoxelExtendedMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let settings = move |s: &mut ImageLoaderSettings| {
        s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::nearest()
        });
    };

    let handle = materials.add(ExtendedMaterial {
        base: asset_server
            .load_with_settings("textures/voxels.png", settings)
            .into(),
        extension: VoxelChunkMaterial { atlas_width: 4 },
    });
    commands.insert_resource(ChunkMaterialRes(handle));
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct VoxelChunkMaterial {
    // Start at a high binding number to ensure bindings don't conflict
    // with the base material
    #[uniform(100)]
    atlas_width: u32,
}

pub const ATTRIBUTE_ATLAS_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("AtlasIndex", 8937522, VertexFormat::Uint32);

impl MaterialExtension for VoxelChunkMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/voxel_chunk.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/voxel_chunk.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_ATLAS_INDEX.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
