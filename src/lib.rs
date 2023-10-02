use gdnative::prelude::*;

mod tables;
mod structs;
mod marching_cubes;
mod terrain_utils;
mod terrain;


fn init(handle: InitHandle) {
    handle.add_tool_class::<terrain::Terrain>();
    handle.add_tool_class::<terrain_utils::TerrainUtils>();
}

godot_init!(init);
