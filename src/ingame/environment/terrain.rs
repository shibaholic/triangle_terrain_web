use std::{collections::HashMap, thread, time::Duration};

use bevy::{color::palettes::css::{BLACK, GREEN, RED, YELLOW}, prelude::*, render::{mesh::{Indices, PrimitiveTopology, VertexAttributeValues}, render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat}, texture::{ImageSampler, ImageSamplerDescriptor}},};
use bevy_fps_controller::controller::LogicalPlayer;
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, RigidBody};
use noise::{core::worley::{distance_functions::euclidean, worley_2d, ReturnType}, permutationtable::PermutationTable, utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder}, Blend, Checkerboard, Fbm, Perlin, RidgedMulti, Vector2};

use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};

use crate::ingame::tricoord::*;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, setup_terrain_assets)
        .init_resource::<TerrainConfig>()
        .init_resource::<Chunks>()
        .register_type::<Chunks>()
        .init_resource::<ChunkTasks>()
        .add_systems(Update, chunks_near_player)
        .add_systems(Update, (begin_generating_chunks, receive_generated_chunks).run_if(run_if_active) )
        ;
    }
}

fn run_if_active(terrain_config: Res<TerrainConfig>) -> bool {
    terrain_config.active
}

#[derive(Resource)]
pub struct TerrainConfig {
    pub chunk_gen_radius:f32,
    pub active:bool,

}

impl Default for TerrainConfig {
    fn default() -> Self {
        TerrainConfig {
            chunk_gen_radius: 20.0,
            active:true
        }
    }
}


fn chunks_near_player(
    query: Query<&Transform, With<LogicalPlayer>>,
    mut chunks: ResMut<Chunks>,
    terrain_config: Res<TerrainConfig>
) /* -> Vec<TriCoord<i16>> */ {
    let player_transform = query.get_single().unwrap();

    let gen_origin = player_transform.translation; 

    let in_radius_tricoords = tricoord_vec_gen_distance(Coord {x:gen_origin.x, z:gen_origin.z}, terrain_config.chunk_gen_radius);

    chunks.in_range = in_radius_tricoords.clone();
}


#[derive(Resource)]
struct TerrainAssetHandles {
    terrain_material_hdls: HashMap<String, Handle<StandardMaterial>>,
    terrain_mesh_hdls: HashMap<String, Handle<Mesh>>,
    height_map_data_hdls: HashMap<Coord<i16>, UntypedHandle>,
    height_map_material_hdls: HashMap<Coord<i16>, Handle<StandardMaterial>>
}
fn setup_terrain_assets(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
    let material_handle_1 = materials.add(Color::srgb(0.1, 0.7, 0.1));
    let material_handle_2 = materials.add(Color::srgb(0.7, 0.2, 0.2));

    let mut mesh = Plane3d::default().mesh().size(16., 16.).build();

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let colors: Vec<[f32; 4]> = positions
            .iter()
            .map(|[r, g, b]| [(1. - *r) / 2., (1. - *g) / 2., (1. - *b) / 2., 1.])
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }

    let mesh_handle = meshes.add(mesh);

    let enviro_asset_handles = TerrainAssetHandles {
        terrain_material_hdls: HashMap::from([("mat_1".into(), material_handle_1), ("mat_2".into(), material_handle_2)]),
        terrain_mesh_hdls: HashMap::from([("chunk_plane".into(), mesh_handle)]),
        height_map_data_hdls: HashMap::new(),
        height_map_material_hdls: HashMap::new()
    };

    commands.insert_resource(enviro_asset_handles);
}


#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions, no_field_bounds)]
pub struct Chunks {
    pub in_range: Vec<TriCoord<i16>>,
    pub generating: Vec<TriCoord<i16>>,
    pub generated: Vec<TriCoord<i16>>
}
impl Default for Chunks {
    fn default() -> Self {
        Self {
            in_range: Vec::new(),
            generating: Vec::new(),
            generated: Vec::new()
        }
    }
}

struct ChunkData {
    tricoord: TriCoord<i16>,
    xy_coord: Coord<f64>,
    mesh: Mesh
}

#[derive(Resource)]
struct ChunkTasks {
    chunk_generation_tasks: HashMap<TriCoord<i16>, Task<ChunkData>>
}
impl Default for ChunkTasks {
    fn default() -> Self {
        Self {
            chunk_generation_tasks: HashMap::new()
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn begin_generating_chunks(
    mut chunks: ResMut<Chunks>,
    mut chunk_tasks: ResMut<ChunkTasks>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    
    for tri_chunk in chunks.in_range.clone() {
        if chunk_tasks.chunk_generation_tasks.contains_key(&tri_chunk) || chunks.generated.contains(&tri_chunk) {
            continue;
        }
        let task = task_pool.spawn(async move {
            create_chunk_data(tri_chunk)
        });
        // println!("started: {} {} {}", tri_chunk.a, tri_chunk.b, tri_chunk.c);
        chunk_tasks.chunk_generation_tasks.insert(tri_chunk.clone(), task);
        chunks.generating.push(tri_chunk);
    }
}

#[cfg(target_arch = "wasm32")]
fn begin_generating_chunks() {

}

#[cfg(not(target_arch = "wasm32"))]
fn receive_generated_chunks(
    mut chunks: ResMut<Chunks>,
    mut chunk_tasks: ResMut<ChunkTasks>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    environ_assets: Res<TerrainAssetHandles>,
    mut commands: Commands,
) {

    // retain keeps the key value pair if true
    chunk_tasks.chunk_generation_tasks.retain(|chunk_coord, task| {
        let status = block_on(future::poll_once(task));

        // is_none means the task is not done, so we retain it.
        let retain = status.is_none();

        // if Some is inside of status (instead of None), then...
        if let Some(data) = status {
            // do actions that are necessary once a chunk is finished generating
            println!("created: {} {} {}", data.tricoord.a, data.tricoord.b, data.tricoord.c);
            
            // convert the data into things that can be spawned
            let terrain_mesh = meshes.add(data.mesh);
            spawn_entities(&data.xy_coord, terrain_mesh, &meshes, &environ_assets, &mut materials, &mut commands);

            chunks.generating.retain(|tricoord| *tricoord != data.tricoord);
            chunks.generated.push(data.tricoord);
        }

        retain
    })
}

#[cfg(target_arch = "wasm32")]
fn receive_generated_chunks(
    mut chunks: ResMut<Chunks>,
    mut chunk_tasks: ResMut<ChunkTasks>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    environ_assets: Res<TerrainAssetHandles>,
    mut commands: Commands,
) {
    for tri_chunk in chunks.in_range.clone() {
        if chunk_tasks.chunk_generation_tasks.contains_key(&tri_chunk) || chunks.generated.contains(&tri_chunk) {
            continue;
        }
        let data = create_chunk_data(tri_chunk);
        let terrain_mesh = meshes.add(data.mesh);
        spawn_entities(&data.xy_coord, terrain_mesh, &meshes, &environ_assets, &mut materials, &mut commands);
        chunks.generated.push(data.tricoord);
    }
}

fn create_chunk_data(
    tricoord: TriCoord<i16>
) -> ChunkData {
    let chunk_coord = trichunk_to_coord(tricoord, 0);
    let noise_map = generate_noise(&tricoord);
    let terrain_mesh = generate_mesh(&tricoord, &noise_map);
    return ChunkData {tricoord, xy_coord: chunk_coord, mesh: terrain_mesh };
}

const PIXEL_BOUND_UNIT:f64 = 1.0/33.0;
fn generate_noise(chunk_tricoord: &TriCoord<i16>) -> NoiseMap {
    let xz = trichunk_to_coord(*chunk_tricoord, 0);
    let halfsides = xz.x / CHUNK_HALFSIDE;

    let lower_x = halfsides * PIXEL_BOUND_UNIT * 16.0;
    let upper_x = lower_x + 1.0;
    
    let upper_y = chunk_tricoord.b as f64 * 32.0 * PIXEL_BOUND_UNIT;
    let lower_y = upper_y - 1.0; 
    
    println!("lower upper x: {},{}", lower_x, upper_x);
    println!("lower upper y: {},{}", lower_y, upper_y);

    let perlin = Perlin::default();
    let ridged = RidgedMulti::<Perlin>::default();
    let fbm = Fbm::<Perlin>::default();
    let blend = Blend::new(perlin, ridged, fbm);

    let noise_map = PlaneMapBuilder::new(blend)
    .set_x_bounds(lower_x, upper_x) // .set_x_bounds(lower_x*2.0, upper_x*2.0)
    .set_y_bounds(lower_y, upper_y)
    .set_size(33, 33) // how many pixels, 33 x 33 pixels
    .build();

    return noise_map;
}

// fn generate_image_material(
//     noise_map: &NoiseMap,
//     images: &mut ResMut<Assets<Image>>,
//     materials: &mut ResMut<Assets<StandardMaterial>>
// ) -> Handle<StandardMaterial> {
//     let (width, height) = noise_map.size();
//     println!("noise_map size: {} {}", width, height);

//     // height map values are from 0 to 255
//     let mut height_map = Vec::with_capacity(width * height);

//     for (index, i) in noise_map.iter().enumerate() {
//         height_map.push(((i * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8);
//     }

//     let mut image = Image::new(
//         Extent3d {
//             width: 16*2+ 1,
//             height: 16*2 + 1,
//             depth_or_array_layers: 1
//         },
//         TextureDimension::D2,
//         height_map.clone(),
//         TextureFormat::R8Unorm,
//         RenderAssetUsages::RENDER_WORLD,
//     );
//     image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());

//     let image_handle = images.add(image);

//     return materials.add(StandardMaterial {
//         base_color_texture: Some(image_handle.clone()),
//         unlit: true,
//         ..default()
//     });
// }


const vertex_assignments_even: [u16; 768] = [
    0,1,17,   1,18,17,   
    1,2,18,   2,19,18,   
    2,3,19,   3,20,19,   
    3,4,20,   4,21,20,   
    4,5,21,   5,22,21,   
    5,6,22,   6,23,22,   
    6,7,23,   7,24,23,   
    7,8,24,   8,25,24,   
    8,9,25,   9,26,25,   
    9,10,26,   10,27,26,   
    10,11,27,   11,28,27,   
    11,12,28,   12,29,28,   
    12,13,29,   13,30,29,   
    13,14,30,   14,31,30,   
    14,15,31,   15,32,31,   
    15,16,32,   17,18,33,   
    18,34,33,   18,19,34,   
    19,35,34,   19,20,35,   
    20,36,35,   20,21,36,   
    21,37,36,   21,22,37,   
    22,38,37,   22,23,38,   
    23,39,38,   23,24,39,   
    24,40,39,   24,25,40,   
    25,41,40,   25,26,41,   
    26,42,41,   26,27,42,   
    27,43,42,   27,28,43,   
    28,44,43,   28,29,44,   
    29,45,44,   29,30,45,   
    30,46,45,   30,31,46,   
    31,47,46,   31,32,47,   
    33,34,48,   34,49,48,   
    34,35,49,   35,50,49,   
    35,36,50,   36,51,50,   
    36,37,51,   37,52,51,   
    37,38,52,   38,53,52,   
    38,39,53,   39,54,53,   
    39,40,54,   40,55,54,   
    40,41,55,   41,56,55,   
    41,42,56,   42,57,56,   
    42,43,57,   43,58,57,   
    43,44,58,   44,59,58,   
    44,45,59,   45,60,59,   
    45,46,60,   46,61,60,   
    46,47,61,   48,49,62,   
    49,63,62,   49,50,63,   
    50,64,63,   50,51,64,   
    51,65,64,   51,52,65,   
    52,66,65,   52,53,66,   
    53,67,66,   53,54,67,   
    54,68,67,   54,55,68,   
    55,69,68,   55,56,69,   
    56,70,69,   56,57,70,   
    57,71,70,   57,58,71,   
    58,72,71,   58,59,72,   
    59,73,72,   59,60,73,   
    60,74,73,   60,61,74,   
    62,63,75,   63,76,75,   
    63,64,76,   64,77,76,   
    64,65,77,   65,78,77,   
    65,66,78,   66,79,78,   
    66,67,79,   67,80,79,   
    67,68,80,   68,81,80,   
    68,69,81,   69,82,81,   
    69,70,82,   70,83,82,   
    70,71,83,   71,84,83,   
    71,72,84,   72,85,84,   
    72,73,85,   73,86,85,   
    73,74,86,   75,76,87,   
    76,88,87,   76,77,88,   
    77,89,88,   77,78,89,   
    78,90,89,   78,79,90,   
    79,91,90,   79,80,91,   
    80,92,91,   80,81,92,   
    81,93,92,   81,82,93,   
    82,94,93,   82,83,94,   
    83,95,94,   83,84,95,   
    84,96,95,   84,85,96,   
    85,97,96,   85,86,97,   
    87,88,98,   88,99,98,   
    88,89,99,   89,100,99,   
    89,90,100,   90,101,100,   
    90,91,101,   91,102,101,   
    91,92,102,   92,103,102,   
    92,93,103,   93,104,103,   
    93,94,104,   94,105,104,   
    94,95,105,   95,106,105,   
    95,96,106,   96,107,106,   
    96,97,107,   98,99,108,   
    99,109,108,   99,100,109,   
    100,110,109,   100,101,110,   
    101,111,110,   101,102,111,   
    102,112,111,   102,103,112,   
    103,113,112,   103,104,113,   
    104,114,113,   104,105,114,   
    105,115,114,   105,106,115,   
    106,116,115,   106,107,116,   
    108,109,117,   109,118,117,   
    109,110,118,   110,119,118,   
    110,111,119,   111,120,119,   
    111,112,120,   112,121,120,   
    112,113,121,   113,122,121,   
    113,114,122,   114,123,122,   
    114,115,123,   115,124,123,   
    115,116,124,   117,118,125,   
    118,126,125,   118,119,126,   
    119,127,126,   119,120,127,   
    120,128,127,   120,121,128,   
    121,129,128,   121,122,129,   
    122,130,129,   122,123,130,   
    123,131,130,   123,124,131,   
    125,126,132,   126,133,132,   
    126,127,133,   127,134,133,   
    127,128,134,   128,135,134,   
    128,129,135,   129,136,135,   
    129,130,136,   130,137,136,   
    130,131,137,   132,133,138,   
    133,139,138,   133,134,139,   
    134,140,139,   134,135,140,   
    135,141,140,   135,136,141,   
    136,142,141,   136,137,142,   
    138,139,143,   139,144,143,   
    139,140,144,   140,145,144,   
    140,141,145,   141,146,145,   
    141,142,146,   143,144,147,   
    144,148,147,   144,145,148,   
    145,149,148,   145,146,149,   
    147,148,150,   148,151,150,   
    148,149,151,   150,151,152,   
    ];

const vertex_assignments_odd: [u16; 768] = [
    0,17,1,   1,17,18,   
    1,18,2,   2,18,19,   
    2,19,3,   3,19,20,   
    3,20,4,   4,20,21,   
    4,21,5,   5,21,22,   
    5,22,6,   6,22,23,   
    6,23,7,   7,23,24,   
    7,24,8,   8,24,25,   
    8,25,9,   9,25,26,   
    9,26,10,   10,26,27,   
    10,27,11,   11,27,28,   
    11,28,12,   12,28,29,   
    12,29,13,   13,29,30,   
    13,30,14,   14,30,31,   
    14,31,15,   15,31,32,   
    15,32,16,   17,33,18,   
    18,33,34,   18,34,19,   
    19,34,35,   19,35,20,   
    20,35,36,   20,36,21,   
    21,36,37,   21,37,22,   
    22,37,38,   22,38,23,   
    23,38,39,   23,39,24,   
    24,39,40,   24,40,25,   
    25,40,41,   25,41,26,   
    26,41,42,   26,42,27,   
    27,42,43,   27,43,28,   
    28,43,44,   28,44,29,   
    29,44,45,   29,45,30,   
    30,45,46,   30,46,31,   
    31,46,47,   31,47,32,   
    33,48,34,   34,48,49,   
    34,49,35,   35,49,50,   
    35,50,36,   36,50,51,   
    36,51,37,   37,51,52,   
    37,52,38,   38,52,53,   
    38,53,39,   39,53,54,   
    39,54,40,   40,54,55,   
    40,55,41,   41,55,56,   
    41,56,42,   42,56,57,   
    42,57,43,   43,57,58,   
    43,58,44,   44,58,59,   
    44,59,45,   45,59,60,   
    45,60,46,   46,60,61,   
    46,61,47,   48,62,49,   
    49,62,63,   49,63,50,   
    50,63,64,   50,64,51,   
    51,64,65,   51,65,52,   
    52,65,66,   52,66,53,   
    53,66,67,   53,67,54,   
    54,67,68,   54,68,55,   
    55,68,69,   55,69,56,   
    56,69,70,   56,70,57,   
    57,70,71,   57,71,58,   
    58,71,72,   58,72,59,   
    59,72,73,   59,73,60,   
    60,73,74,   60,74,61,   
    62,75,63,   63,75,76,   
    63,76,64,   64,76,77,   
    64,77,65,   65,77,78,   
    65,78,66,   66,78,79,   
    66,79,67,   67,79,80,   
    67,80,68,   68,80,81,   
    68,81,69,   69,81,82,   
    69,82,70,   70,82,83,   
    70,83,71,   71,83,84,   
    71,84,72,   72,84,85,   
    72,85,73,   73,85,86,   
    73,86,74,   75,87,76,   
    76,87,88,   76,88,77,   
    77,88,89,   77,89,78,   
    78,89,90,   78,90,79,   
    79,90,91,   79,91,80,   
    80,91,92,   80,92,81,   
    81,92,93,   81,93,82,   
    82,93,94,   82,94,83,   
    83,94,95,   83,95,84,   
    84,95,96,   84,96,85,   
    85,96,97,   85,97,86,   
    87,98,88,   88,98,99,   
    88,99,89,   89,99,100,   
    89,100,90,   90,100,101,   
    90,101,91,   91,101,102,   
    91,102,92,   92,102,103,   
    92,103,93,   93,103,104,   
    93,104,94,   94,104,105,   
    94,105,95,   95,105,106,   
    95,106,96,   96,106,107,   
    96,107,97,   98,108,99,   
    99,108,109,   99,109,100,   
    100,109,110,   100,110,101,   
    101,110,111,   101,111,102,   
    102,111,112,   102,112,103,   
    103,112,113,   103,113,104,   
    104,113,114,   104,114,105,   
    105,114,115,   105,115,106,   
    106,115,116,   106,116,107,   
    108,117,109,   109,117,118,   
    109,118,110,   110,118,119,   
    110,119,111,   111,119,120,   
    111,120,112,   112,120,121,   
    112,121,113,   113,121,122,   
    113,122,114,   114,122,123,   
    114,123,115,   115,123,124,   
    115,124,116,   117,125,118,   
    118,125,126,   118,126,119,   
    119,126,127,   119,127,120,   
    120,127,128,   120,128,121,   
    121,128,129,   121,129,122,   
    122,129,130,   122,130,123,   
    123,130,131,   123,131,124,   
    125,132,126,   126,132,133,   
    126,133,127,   127,133,134,   
    127,134,128,   128,134,135,   
    128,135,129,   129,135,136,   
    129,136,130,   130,136,137,   
    130,137,131,   132,138,133,   
    133,138,139,   133,139,134,   
    134,139,140,   134,140,135,   
    135,140,141,   135,141,136,   
    136,141,142,   136,142,137,   
    138,143,139,   139,143,144,   
    139,144,140,   140,144,145,   
    140,145,141,   141,145,146,   
    141,146,142,   143,147,144,   
    144,147,148,   144,148,145,   
    145,148,149,   145,149,146,   
    147,150,148,   148,150,151,   
    148,151,149,   150,152,151,   
    ];

// make the res mut. add the height map data so it can create the mesh
fn generate_mesh(
    chunk_tricoord: &TriCoord<i16>,
    noise_map: &NoiseMap
) -> Mesh {
    let odd:bool = chunk_tricoord.a + chunk_tricoord.b + chunk_tricoord.c != 0;
    let vertices = generate_vertices(noise_map, odd);
    let mut normals = Vec::with_capacity(triangular_number_o1(CHUNK_SIDE as i16 + 1) as usize);
    for i in 0..normals.capacity() {
        normals.push([0.0, 1.0, 0.0]);
    }
    let assignments = if odd {
        vertex_assignments_odd.to_vec()
    } else {
        // even
        vertex_assignments_even.to_vec()
    };

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertices
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        normals
    )
    .with_inserted_indices(Indices::U16(assignments));

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let colors: Vec<[f32; 4]> = positions
            .iter()
            .map(|[r, g, b]| [(1. - *r) , (1. - *g) , (1. - *b) , 1.])
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    
    return mesh; 
}


const HEIGHT_AMPLIFIER:f64 = 5.0;
fn generate_vertices(noise_map: &NoiseMap, odd:bool) -> Vec<Vec3> {
    let mut height_grid:[[f64;33]; 33] = [[0.0; 33]; 33]; // row, col. dont confuse with x,y coordinates
    //println!("\n height grid");
    for (index, value) in noise_map.iter().enumerate() {
        height_grid[index / 33][index % 33] = *value;
        //println!("{} {} = {}", index2 / 33, index2 % 33, *value);
    }

    let mut v: Vec<Vec3> = Vec::with_capacity(triangular_number_o1(CHUNK_SIDE as i16) as usize);

    let origin_offset_x = CHUNK_SIDE as f64/2.0 * TRI_SIDE as f64;
    let origin_offset_y = -CHUNK_SIDE as f64/2.0 * TRI_ALTITUDE as f64;
    //println!("origin_x_z: {} {}", origin_offset_x, origin_offset_y);

    for row_index in 0..(CHUNK_SIDE as i16 + 1) {
        
        let x_offset = row_index as f64 * (TRI_SIDE/2.0) as f64;

        let col_index_max = CHUNK_SIDE as i16 - row_index;
        let mut noise_map_x_start = row_index;
        for col_index in 0..col_index_max + 1 {
            let x = (x_offset + col_index as f64 * TRI_SIDE as f64) - origin_offset_x;
            
            let z = if odd {
                (row_index as f64 * TRI_ALTITUDE as f64) + origin_offset_y
            } else {
                // even
                (row_index as f64 * -TRI_ALTITUDE as f64) - origin_offset_y
            };

            let noise_max = 32;

            let y = if odd {
                height_grid[(row_index*2) as usize][(noise_map_x_start+col_index*2) as usize]
            } else {
                // even
                height_grid[(noise_max-row_index*2) as usize][(noise_map_x_start+col_index*2) as usize]
            } * HEIGHT_AMPLIFIER;
            v.push( Vec3 {x:x as f32, y:y as f32, z:z as f32} );
        }
    }
    return v;
}


fn spawn_entities(
    chunk_coord: &Coord<f64>, 
    terrain_mesh: Handle<Mesh>,
    meshes: &Assets<Mesh>,
    environ_assets: &TerrainAssetHandles,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands
) {
    let middle_x = chunk_coord.x;
    let middle_y = chunk_coord.z;

    // terrain transform from ChunkCoord
    let chunk_transform = Transform {
        translation: Vec3::new(middle_x as f32, 0., middle_y as f32),
        ..default()
    };
    // terrain collider
    let terrain_collider = Collider::from_bevy_mesh(meshes.get(&terrain_mesh).unwrap(), &ComputedColliderShape::TriMesh).unwrap();

    // spawn terrain
    commands.spawn((
        PbrBundle {
            mesh: terrain_mesh.clone(),
            material: materials.add(Color::srgb(1., 1., 1.)) ,
            transform: chunk_transform,
            ..default()
        },
        terrain_collider,
        RigidBody::Fixed,
    ))
    .insert(Name::new("TerrainMesh"));
}