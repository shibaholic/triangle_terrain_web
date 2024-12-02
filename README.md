
<br />
<div align="center">
  <a href="https://github.com/othneildrew/Best-README-Template">
    <img src="documentation/images/triangle_terrain.png" alt="Logo" width="160" height="150">
  </a>

  <h3 align="center">Triangle_Terrain_Web</h3>
</div>

# Triangular tiling mesh for procedurally generated terrain

This project attempts to create procedurally generated terrain using a height map of triangular tilings, instead of the more common square tiling. 

Watch the demo video on YouTube [here](https://youtube.com/watch?v=KJ-jy8l7Py0).

In browser WebGL version playable [here](https://shibaholic.github.io/triangle_terrain_web).

| |
|-|
|![Screenshot of triangle tiling terrain](/documentation/images/result.png)
| *Project result; showing triangle tiling terrain.* |


# Why

I was wondering if low poly terrain would look better using triangle tiling or with square tiling. 

| |
|-|
|![Screenshot of square tiling terrain](/documentation/images/low_poly_terrain.png)
| *Example of low poly terrain using square tiling. (Not my work)*|

# Technologies

Rust and the Bevy game engine was used.

# Contents of the project

- Procedural generation for triangle tiling terrain
    - Noise map using [noise.rs](https://crates.io/crates/noise)
    - Multiple coordinate systems (a,b,c tri coord, 2x-1y coord, xy coord)
    - Entire chunk mesh calculated at runtime (inefficient, but oh well)
    - Generation based on distance to player
- 2 different terrain materials
- FPS controller for flying and walking (using [bevy_fps_controller](https://crates.io/crates/bevy_fps_controller))
- Debug menu (using [bevy_egui](https://github.com/vladbat00/bevy_egui))

# Development screenshots

| |
|-|
|![Screenshot of development](/documentation/images/development/1.png)
|*1. Visualizing the placement of each chunk. Material color is vertex coordinate applied as vertex color. The white outlines of the triangles are the individual mesh triangles.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/2.png)
|*2. Odd and even chunks.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/3.png)
|*3. Testing of applying height map (bottom red and black tiles) to mesh.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/4.png)
|*4. The height map is not being properly sampled resulting in bad seams between the chunks.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/5.png)
|*5. Testing generating a lot of chunks.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/6.png)
|*6. Worley noise height map.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/7.png)
|*7. Perlin noise height map.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/8.png)
|*8. Testing terrain generation distance from player. Red boxes are chunks that are within the radius.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/9.png)
|*9. Discovering that the meshes were not being built in the correct direction (which affects PBR and normal maps).*|

| |
|-|
|![Screenshot of development](/documentation/images/development/10.png)
|*10. Using green for even and shiny for odd chunks. The "skybox" environment map is visible in the shiny reflection.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/11.png)
|*11. A lot of terrain using green shiny material.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/13.png)
|*12. Terrain from up and above.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/14.png)
|*13. All shiny terrain.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/15.png)
|*14. Applying shininess to terrain with vertex color from vertex coordinates.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/16.png)
|*15. It looks like candy.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/17.png)
|*16. Another view.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/18.png)
|*17. Using mesh tri visualizer.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/19.png)
|*18. Without.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/20.png)
|*19. Reflection visible.*|

| |
|-|
|![Screenshot of development](/documentation/images/development/21.png)
|*20. Changed noise generation to large scale and created a custom WGSL vertex shader that changes the vertex color to gray on slopes.*|