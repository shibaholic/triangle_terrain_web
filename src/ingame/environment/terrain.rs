use std::{any::TypeId, collections::HashMap, thread, time::Duration};

use bevy::{render::render_resource::{AsBindGroup, ShaderRef}, color::palettes::css::{BLACK, GREEN, RED, YELLOW}, pbr::{ExtendedMaterial, MaterialExtension}, prelude::*, render::{mesh::{Indices, PrimitiveTopology, VertexAttributeValues}, render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat}, texture::{ImageSampler, ImageSamplerDescriptor}}};
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
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, MyMaterial>,>::default())
        .add_systems(Startup, setup_terrain_assets)
        .insert_resource(SelectedTerrainMat {
            selected_mat: "my_mat".into()
        })
        .register_type::<SelectedTerrainMat>()
        .init_resource::<TerrainConfig>()
        .init_resource::<Chunks>()
        .register_type::<Chunks>()
        .init_resource::<ChunkTasks>()
        .add_systems(Update, chunks_near_player)
        .add_systems(Update, (begin_generating_chunks, receive_generated_chunks).run_if(run_if_terrain_active) )
        ;
    }
}

fn run_if_terrain_active(terrain_config: Res<TerrainConfig>) -> bool {
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

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct MyMaterial {}

impl MaterialExtension for MyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/animate_shader.wgsl".into()
    }
}

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct SelectedTerrainMat {
    pub selected_mat: String
}

#[derive(Resource)]
pub struct TerrainHandles {
    pub mat_hdls: HashMap<String, UntypedHandle>,
    mesh_hdls: HashMap<String, Handle<Mesh>>,
    height_map_hdls: HashMap<Coord<i16>, UntypedHandle>
}
fn setup_terrain_assets(
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut mymat_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MyMaterial>>>,
    mut commands: Commands
) {
    let shiny_material = standard_materials.add(
        StandardMaterial {
            // base_color: Srgba::hex("#6dbe4b").unwrap().into(),
            base_color: Color::srgb(0.5, 0.5, 0.5),
            metallic: 1.0,
            perceptual_roughness: 0.0,
            reflectance: 1.0,
            ..default()
    });

    let my_material = mymat_materials.add(
        ExtendedMaterial {
            base: StandardMaterial {
                // base_color: Srgba::hex("#6dbe4b").unwrap().into(),
                base_color: Color::srgb(0.5, 0.5, 0.5),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                ..default()
            },
            extension: MyMaterial {  }
        }
    );

    // let mut mesh = Plane3d::default().mesh().size(16., 16.).build();

    // if let Some(VertexAttributeValues::Float32x3(positions)) =
    //     mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    // {
    //     let colors: Vec<[f32; 4]> = positions
    //         .iter()
    //         .map(|[r, g, b]| [(1. - *r) / 2., (1. - *g) / 2., (1. - *b) / 2., 1.])
    //         .collect();
    //     // mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    // }

    // let mesh_handle = meshes.add(mesh);

    let terrain_hdls = TerrainHandles {
        mat_hdls: HashMap::from([("shiny".into(), shiny_material.untyped()), ("my_mat".into(), my_material.untyped())]),
        mesh_hdls: HashMap::from([/*("chunk_plane".into(), mesh_handle)*/]),
        height_map_hdls: HashMap::new()
    };

    commands.insert_resource(terrain_hdls);
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
    environ_assets: Res<TerrainHandles>,
    mut commands: Commands,
    selected_mat: Res<SelectedTerrainMat>
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
            spawn_terrain(&data.xy_coord, terrain_mesh, &meshes, &environ_assets, &selected_mat, &mut commands, );

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
    environ_assets: Res<TerrainHandles>,
    mut commands: Commands,
) {
    for tri_chunk in chunks.in_range.clone() {
        if chunk_tasks.chunk_generation_tasks.contains_key(&tri_chunk) || chunks.generated.contains(&tri_chunk) {
            continue;
        }
        let data = create_chunk_data(tri_chunk);
        let terrain_mesh = meshes.add(data.mesh);
        spawn_terrain(&data.xy_coord, terrain_mesh, &meshes, &environ_assets, &mut materials, &mut commands);
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

const BOUND_FACTOR:f64 = 0.05;
const PIXEL_BOUND_UNIT:f64 = BOUND_FACTOR/33.0;
fn generate_noise(chunk_tricoord: &TriCoord<i16>) -> NoiseMap {
    let xz = trichunk_to_coord(*chunk_tricoord, 0);
    let halfsides = xz.x / CHUNK_HALFSIDE;

    let lower_x = halfsides * 16.0 * PIXEL_BOUND_UNIT;
    let upper_x = lower_x + BOUND_FACTOR;
    
    let upper_y = chunk_tricoord.b as f64 * 32.0 * PIXEL_BOUND_UNIT;
    let lower_y = upper_y - BOUND_FACTOR; 
    
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

// make the res mut. add the height map data so it can create the mesh
fn generate_mesh(
    chunk_tricoord: &TriCoord<i16>,
    noise_map: &NoiseMap
) -> Mesh {
    let odd:bool = chunk_tricoord.a + chunk_tricoord.b + chunk_tricoord.c != 0;
    let vertices = generate_vertices_3s(noise_map, odd); // 768/3 = 256

    let normals = generate_normals_from_trimesh(&vertices, odd);

    let assignments = generate_assignments(odd);

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

    // r min max = -7, 9
    // g min max = -1.2, 4
    // b min max = -6, 8
    let min = 0.0;

    let clamp_min = |value: f32, min| {
        if value < min {
            min
        } else {
            value
        }
    };

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let colors: Vec<[f32; 4]> = positions
            .iter()
            .map(|[x, y, z]| {
                // [(1. - *r).abs(), (1. - *g).abs(), (1. - *b).abs(), 1.]
                let mut r = clamp_min(1. - *z, min);
                let mut g = clamp_min((1. - *x), min);
                let mut b = clamp_min(1. + *x, min);
                // if r + g + b > 20.0 {
                //     g = 15.0;
                // }
                // println!("rgb: {} {} {}", r, g, b);
                [r, g, b, 1.]
                // [5.0, 5.0, 0.0, 1.]
            } )
            .collect();

        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    
    return mesh; 
}

const NOISE_MIN:f32 = 0.0;
const NOISE_MAX:f32 = 1.0;
const HEIGHT_MIN:f32 = 0.0;
const HEIGHT_MAX:f32 = 100.0;
fn map_noise_to_height(value: f32) -> f32 {
    HEIGHT_MIN + (value - NOISE_MIN) * (HEIGHT_MAX - HEIGHT_MIN) / (NOISE_MAX - NOISE_MIN)
}

fn generate_vertices_3s(noise_map: &NoiseMap, odd:bool) -> Vec<Vec3> {
    let mut v:Vec<Vec3> = Vec::with_capacity(triangular_number_o1(CHUNK_SIDE) as usize * 3);

    // convert 1-dimensional noise_map into a grid
    let mut noise_grid:[[f64;33]; 33] = [[0.0; 33]; 33]; // row v, col ->. dont confuse with x,y coordinates
    for (index, value) in noise_map.iter().enumerate() {
        noise_grid[index / 33][index % 33] = *value;
    }

    //let test_x = 1;
    //let test_y = 30;
    //let print_test = |y:usize, x:usize, s:String| { println!("{} noise[{}][{}]: {}", s, y, x, (noise_grid[y][x] * HEIGHT_AMPLIFIER as f64)); };
    // print_test(test_y, test_x, String::from("TEST"));

    // origin offsets, so the mesh origin is in the correct center (where ALTITUDE on the z-axis and SIDE on the x-axis crosses, so not the geometric center)
    let origin_offset_x = CHUNK_HALFSIDE * TRI_SIDE as f64;
    // let origin_offset_z = -CHUNK_HALFSIDE * TRI_ALTITUDE as f64;

    let (z_alt, z_halfalt, z_noise_augmenter, z_noise_start, origin_offset_z) = if !odd {
        // even chunk
        (-TRI_ALTITUDE,
        -TRI_HALF_ALT,
        -2,
        32,
        -CHUNK_HALFSIDE * TRI_ALTITUDE as f64)
    } else {
        // odd chunk
        (TRI_ALTITUDE,
        TRI_HALF_ALT,
        2,
        0,
        CHUNK_HALFSIDE * TRI_ALTITUDE as f64)
    };

    // loop through each row of triangles in a chunk
    for row_index in 0..CHUNK_SIDE { // CHUNK_SIDE
        let x_row_offset = row_index as f32 * TRI_HALFSIDE + TRI_HALFSIDE;

        // loop through each individual triangle (including odd triangles) in a row
        let col_max = ((CHUNK_SIDE - (row_index + 1)) * 2) + 1;
        for col_index in 0..col_max {
            let z_col_offset = (row_index as f32 * z_alt) + z_alt/2.0;
            // x_offset and z_offset now point to the centre of each trianglet

            let x_base = -origin_offset_x as f32 + x_row_offset + col_index as f32 * TRI_HALFSIDE;
            let z_base = -origin_offset_z as f32 + z_col_offset;

            // x and z noise_base point to the left vertex of each trianglet pixel in the noise_map
            let x_noise_base = row_index + col_index;
            let z_noise_base = z_noise_start + z_noise_augmenter * row_index;

            // println!("");

            let mut print_and_push = |vec3| { 
                // println!("{:?}",vec3); 
                //println!("z_col_offset: {}", z_col_offset);
                v.push(vec3); 
            };

            //println!("z_base: {}", z_base);

            // println!("  x_z_noise_base: {}, {}", x_noise_base, z_noise_base);

            if col_index % 2 == 0 {
                // even trianglet
                
                // print_test( z_noise_base as usize, x_noise_base as usize, String::from("EVEN LEFT"));

                // left vertex
                print_and_push(Vec3 {
                    x:x_base - TRI_HALFSIDE, 
                    y:map_noise_to_height(noise_grid[z_noise_base as usize][x_noise_base as usize] as f32), 
                    z:z_base - z_halfalt
                });

                // right vertex
                print_and_push(Vec3 {
                    x:x_base + TRI_HALFSIDE, 
                    y:map_noise_to_height(noise_grid[z_noise_base as usize][(x_noise_base + 2) as usize] as f32), 
                    z:z_base - z_halfalt
                });

                // print_test( (z_noise_base + z_noise_augmenter) as usize, (x_noise_base + 1) as usize, String::from("EVEN ALTI"));

                // altitude vertex
                print_and_push(Vec3 {
                    x:x_base, 
                    y:map_noise_to_height(noise_grid[(z_noise_base + z_noise_augmenter) as usize][(x_noise_base + 1) as usize] as f32), 
                    z:z_base + z_halfalt
                });
            } else {
                // odd trianglet

                // print_test( (z_noise_base) as usize, (x_noise_base + 1) as usize, String::from("ODD  ALTI"));

                // altitude vertex
                print_and_push(Vec3 {
                    x:x_base, 
                    y:map_noise_to_height(noise_grid[(z_noise_base) as usize][(x_noise_base + 1) as usize] as f32), 
                    z:z_base - z_halfalt
                });
                // right vertex
                print_and_push(Vec3 {
                    x:x_base + TRI_HALFSIDE, 
                    y:map_noise_to_height(noise_grid[(z_noise_base + z_noise_augmenter) as usize][(x_noise_base + 2) as usize] as f32), 
                    z:z_base + z_halfalt
                });

                // print_test( (z_noise_base + z_noise_augmenter) as usize, x_noise_base as usize, String::from("ODD  LEFT"));

                // left vertex
                print_and_push(Vec3 {
                    x:x_base - TRI_HALFSIDE, 
                    y:map_noise_to_height(noise_grid[(z_noise_base + z_noise_augmenter) as usize][x_noise_base as usize] as f32), 
                    z:z_base + z_halfalt
                });
            }
        }
    }

    return v;
}

// takes in a flat array of vertices that are triangles and creates a flat array of normals for each vertex in flat shading
fn generate_normals_from_trimesh(vertices: &Vec<Vec3>, odd:bool) -> Vec<Vec3> {
    let mut v:Vec<Vec3> = Vec::with_capacity(vertices.len());

    if !odd {
        // even
        for index in 0..vertices.len()/3 {
            v.push(calculate_normal(vertices[index*3], vertices[index*3+1], vertices[index*3+2]));
            v.push(calculate_normal(vertices[index*3], vertices[index*3+1], vertices[index*3+2]));
            v.push(calculate_normal(vertices[index*3], vertices[index*3+1], vertices[index*3+2]));
        }
    } else {
        // odd
        for index in 0..vertices.len()/3 {
            v.push(calculate_normal(vertices[index*3+1], vertices[index*3], vertices[index*3+2]));
            v.push(calculate_normal(vertices[index*3+1], vertices[index*3], vertices[index*3+2]));
            v.push(calculate_normal(vertices[index*3+1], vertices[index*3], vertices[index*3+2]));
        }
    }


    return v;
}

fn calculate_normal(v0: Vec3, v1: Vec3, v2: Vec3) -> Vec3 {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;

    let normal = edge1.cross(edge2);

    normal.normalize()
}

fn generate_assignments(odd: bool) -> Vec<u16> {
    let size = (CHUNK_SIDE.pow(2) * 3) as usize;
    let mut v: Vec<u16> = Vec::with_capacity(size);
    if !odd {
        // even chunk
        for index in 0..(size) {
            v.push(index as u16);
        }
    } else {
        // odd chunk
        for index in 0..size/3 {
            v.push((index * 3 + 1) as u16);
            v.push((index * 3) as u16);
            v.push((index * 3 + 2) as u16);
        }
    }


    return v;
}

#[derive(Component)]
pub struct TerrainMesh {}

fn spawn_terrain(
    chunk_coord: &Coord<f64>, 
    terrain_mesh: Handle<Mesh>,
    meshes: &Assets<Mesh>,
    environ_assets: &TerrainHandles,
    selected_mat: &SelectedTerrainMat,
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

    // println!("selected mat: {}", selected_mat.selected_mat);
    // println!("selected id: {:?}", environ_assets.mat_hdls[&selected_mat.selected_mat].clone().type_id());
    // println!("shiny id: {:?}", environ_assets.mat_hdls["shiny"].clone().type_id());

    if selected_mat.selected_mat == "shiny" {
        println!("spawn shiny");
        // spawn terrain
        commands.spawn((
            PbrBundle {
                mesh: terrain_mesh.clone(),
                material: environ_assets.mat_hdls["shiny"].clone().typed_unchecked(), // materials.add(Color::srgb(1., 1., 1.)) ,
                transform: chunk_transform,
                ..default()
            },
            terrain_collider,
            RigidBody::Fixed,
            TerrainMesh {},
        ))
        .insert(Name::new("TerrainMesh"));
    } else if selected_mat.selected_mat == "my_mat" {
        println!("spawn my_mat");
            // spawn terrain
    commands.spawn((
        MaterialMeshBundle {
            mesh: terrain_mesh.clone(),
            material: environ_assets.mat_hdls.get("my_mat").unwrap().clone().typed::<ExtendedMaterial<StandardMaterial, MyMaterial>>(), // materials.add(Color::srgb(1., 1., 1.)) ,
            transform: chunk_transform,
            ..default()
        },
        terrain_collider,
        RigidBody::Fixed,
        TerrainMesh {},
    ))
    .insert(Name::new("TerrainMesh"));
    }


}