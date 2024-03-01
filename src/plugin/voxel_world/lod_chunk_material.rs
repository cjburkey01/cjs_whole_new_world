use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexFormat,
        },
    },
};

pub struct LodChunkMaterialPlugin;

impl Plugin for LodChunkMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<LodChunkExtendedMaterial>::default())
            .add_systems(Startup, add_lod_chunk_material_system);
    }
}

pub type LodChunkExtendedMaterial = ExtendedMaterial<StandardMaterial, LodChunkMaterial>;

#[derive(Resource)]
pub struct LodChunkMaterialRes(pub Handle<LodChunkExtendedMaterial>);

fn add_lod_chunk_material_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<LodChunkExtendedMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            metallic: 0.01,
            reflectance: 0.02,
            double_sided: true,
            ..default()
        },
        // is there a way to use an extension without any extra uniforms??
        extension: default(),
    });
    commands.insert_resource(LodChunkMaterialRes(handle));
}

#[derive(Default, Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct LodChunkMaterial {
    #[uniform(100)]
    dummy: u32,
}

// TODO: REDO VERTEX FORMAT TO INCLUDE VOXEL COLOR
pub const ATTRIBUTE_LOD_HACK_VERT: MeshVertexAttribute =
    MeshVertexAttribute::new("HackLodVertAss", 989247824257, VertexFormat::Uint32x2);

impl MaterialExtension for LodChunkMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/lod_chunk.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/lod_chunk.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            // TODO: FIND A WAY TO STOP PROVIDING THE POSITION.
            //       I DON'T WANT TO HAVE TO MAKE A COPY OF HALF THE PBR PIPELINE.
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_LOD_HACK_VERT.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
