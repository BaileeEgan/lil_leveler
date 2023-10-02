use super::tables::*;
use super::structs::*;
use gdnative::prelude::Vector3;

const EMPTY_TRI:Triangle = Triangle::new(
    Vector3::new(0.0, 0.0, 0.0),
    Vector3::new(0.0, 0.0, 0.0),
    Vector3::new(0.0, 0.0, 0.0)
);



const ADJACENT:[(u8,u8);3] = [
    (1, 0),
    (0, 1),
    (1, 1)
];

pub fn generate_chunk_mesh_from_heightmap (heights:&Vec<i32>, chunk_id:usize, chunk_size:usize, num_chunks:usize, lod_list:&Vec<usize>) -> Vec<Vector3> {
    let mut verts_out:Vec<Vector3> = Vec::new();
    
    let chunk_col:usize = chunk_id % num_chunks;
    let chunk_row:usize = chunk_id / num_chunks;
    let offset_x = chunk_col * chunk_size;
    let offset_z = chunk_row * chunk_size;
    let map_size = chunk_size * num_chunks;

    let transition_top = chunk_row > 0 && lod_list[(chunk_row - 1) * num_chunks as usize + chunk_col] > lod_list[chunk_id];
    let transition_bottom = chunk_row < num_chunks as usize - 1 && lod_list[(chunk_row + 1) * num_chunks as usize + chunk_col] > lod_list[chunk_id];

    let transition_left = chunk_col > 0 && lod_list[chunk_row * num_chunks as usize + chunk_col - 1] >  lod_list[chunk_id];
    let transition_right = chunk_col < num_chunks as usize - 1 && lod_list[chunk_row * num_chunks as usize + chunk_col + 1] > lod_list[chunk_id];

    let get_height_range = |current_height:i32, x:usize, z:usize| -> (i32, i32) {
        let mut max_height:i32 = current_height;
        let mut min_height:i32 = current_height;
        for dir in ADJACENT {
            let x_min:usize = (x + dir.0 as usize).min(chunk_size * num_chunks - 1);
            let z_min:usize = (z + dir.1 as usize).min(chunk_size * num_chunks - 1);
            let adjacent_height = heights[z_min * map_size + x_min];
            max_height = max_height.max(adjacent_height);
            min_height = min_height.min(adjacent_height);
        }
        return (min_height,max_height);
    };

    for x in 0..chunk_size {
        for z in 0..chunk_size {
            let global_x = x + offset_x;
            let global_z = z + offset_z;
            let current_height = heights[global_z * map_size + global_x];
            let height_range = get_height_range(current_height, global_x, global_z);
            
            for y in height_range.0..=height_range.1 {
                let mut densities:u8 = 0b00000000;

                for i in 0..8 as usize {
                    let x_min:usize = (global_x + POINTS[i].x as usize).min(chunk_size * num_chunks - 1);
                    let z_min:usize = (global_z + POINTS[i].z as usize).min(chunk_size * num_chunks - 1);
                    let height:i32 = heights[z_min * map_size + x_min];
                    if y + POINTS[i].y as i32 <= height {
                        densities |= 1 << i;
                    }
                }
                let triangles:[Triangle; 5] = march(densities);

                for triangle in triangles.iter() {
                    if *triangle == EMPTY_TRI {
                        break;
                    }
                    else {
                        for point in triangle.points.iter() {
                            let mut vert = *point + Vector3::new(x as f32, y as f32, z as f32);
                            
                            let t_top = height_range.1 != height_range.0 && transition_top && vert.z as i32 == 0 && vert.x != vert.x.trunc();
                            let t_bottom = height_range.1 != height_range.0 && transition_bottom && vert.z as i32 == chunk_size as i32 && vert.x != vert.x.trunc();
                            let t_left = height_range.1 != height_range.0 && transition_left && vert.x as i32 == 0 && vert.z != vert.z.trunc();
                            let t_right = height_range.1 != height_range.0 && transition_right && vert.x as i32 == chunk_size as i32 && vert.z != vert.z.trunc();
                            
                            if t_top || t_bottom || t_left || t_right {
                                vert.y = 0.5 * height_range.0 as f32 + 0.5 * height_range.1 as f32 + 0.5;
                            }


                            verts_out.push(vert);
                        }
                    }
                }
            }
            
        }
    }
    return verts_out;
}

pub fn generate_chunk_mesh_from_height_array (heights:&Vec<i32>, chunk_id:usize, chunk_size:usize, num_chunks:usize) -> Vec<Vector3> {
    let mut verts_out:Vec<Vector3> = Vec::new();
    
    let chunk_col:usize = chunk_id % num_chunks;
    let chunk_row:usize = chunk_id / num_chunks;
    let offset_x = chunk_col * chunk_size;
    let offset_z = chunk_row * chunk_size;

    for x in 0..chunk_size {
        for z in 0..chunk_size {
            let global_x = x + offset_x;
            let global_z = z + offset_z;
            let current_height = heights[global_z * chunk_size * num_chunks + global_x] as usize;
            let mut max_height = current_height;
            let mut min_height = current_height;

            for dir in ADJACENT {
                let x_min:usize = (global_x + dir.0 as usize).min(chunk_size * num_chunks - 1);
                let z_min:usize = (global_z + dir.1 as usize).min(chunk_size * num_chunks - 1);
                let adjacent_height = heights[z_min * chunk_size * num_chunks + x_min] as usize;
                max_height = max_height.max(adjacent_height);
                min_height = min_height.min(adjacent_height);
            }
            
            for y in min_height..=max_height {
                let mut densities:u8 = 0b00000000;

                for i in 0..8 as usize {
                    let x_min:usize = (global_x + POINTS[i].x as usize).min(chunk_size * num_chunks - 1);
                    let z_min:usize = (global_z + POINTS[i].z as usize).min(chunk_size * num_chunks - 1);
                    let height:i32 = heights[z_min * chunk_size * num_chunks + x_min];
                    if y + POINTS[i].y as usize <= height as usize {
                        densities |= 1 << i;
                    }
                }
                let triangles:[Triangle; 5] = march(densities);

                for triangle in triangles.iter() {
                    if *triangle == EMPTY_TRI {
                        break;
                    }
                    else {
                        for point in triangle.points.iter() {
                            verts_out.push(*point + Vector3::new(x as f32, y as f32, z as f32));
                        }
                    }
                }
            }
            
        }
    }
    return verts_out;
}


pub fn generate_chunk_lod_mesh (heights:&Vec<i32>, chunk_id:usize, chunk_size:usize, num_chunks:usize, lod_list:&Vec<usize>, indices_out:&mut Vec<usize>, vertices_out:&mut Vec<Vector3>) {
    let mut verts_out:Vec<Vector3> = Vec::new();

    let chunk_col:usize = chunk_id % num_chunks;
    let chunk_row:usize = chunk_id / num_chunks;
    let offset_x = chunk_col * chunk_size;
    let offset_z = chunk_row * chunk_size;

    let transition_top = chunk_row > 0 && lod_list[(chunk_row - 1) * num_chunks as usize + chunk_col] > lod_list[chunk_id];
    let transition_bottom = chunk_row < num_chunks as usize - 1 && lod_list[(chunk_row + 1) * num_chunks as usize + chunk_col] > lod_list[chunk_id];

    let transition_left = chunk_col > 0 && lod_list[chunk_row * num_chunks as usize + chunk_col - 1] >  lod_list[chunk_id];
    let transition_right = chunk_col < num_chunks as usize - 1 && lod_list[chunk_row * num_chunks as usize + chunk_col + 1] > lod_list[chunk_id];


    let lod = lod_list[chunk_id] - 1;
    let step = 1 << lod;
    let lod_chunk_size = chunk_size / step;
    let map_size = chunk_size * num_chunks;

    for z in 0..(lod_chunk_size + 1) {
        for x in 0..(lod_chunk_size + 1) {
            let global_x = (x * step + offset_x).min(map_size - 1);
            let global_z = (z * step + offset_z).min(map_size - 1);
            let current_height = heights[global_z * map_size + global_x];

            let new_vertex:Vector3 = Vector3::new(
                (x * step) as f32,
                current_height as f32 + 0.5, // Add 0.5 because marching cubes adds faces between cells, not on cell borders
                (z * step) as f32
            );
            vertices_out.push(new_vertex);
        }
    }

    if transition_top || transition_bottom || transition_left || transition_right {
        if transition_top {
            let z = 0;
            for x in (1..(lod_chunk_size)).step_by(2) {
                let i  = z * (lod_chunk_size ) + x;
                let left_vert = vertices_out[i - 1];
                let right_vert = vertices_out[i + 1];
                vertices_out[i].y = 0.5 * (left_vert.y + right_vert.y);
            }
        }
    
       if transition_bottom {
            let z = lod_chunk_size;
            for x in (1..(lod_chunk_size)).step_by(2) {
                let i  = z * (lod_chunk_size + 1) + x;
                let left_vert = vertices_out[i - 1];
                let right_vert = vertices_out[i + 1];
                vertices_out[i].y = 0.5 * (left_vert.y + right_vert.y);
            }
        }
    
        if transition_left {
            let x = 0;
            for z in (1..lod_chunk_size).step_by(2) {
                let i  = z * (lod_chunk_size + 1) + x;
                let top_vert = vertices_out[i - lod_chunk_size - 1];
                let bottom_vert = vertices_out[i + lod_chunk_size + 1];
                vertices_out[i].y = 0.5 * (top_vert.y + bottom_vert.y);
            }
        }
    
        if transition_right {
            let x = lod_chunk_size;
            for z in (1..lod_chunk_size).step_by(2) {
                let i  = z * (lod_chunk_size + 1) + x;
                let top_vert = vertices_out[i - lod_chunk_size - 1];
                let bottom_vert = vertices_out[i + lod_chunk_size + 1];
                vertices_out[i].y = 0.5 * (top_vert.y + bottom_vert.y);
            }
        }
    }

    for z in 0..lod_chunk_size {
        for x in 0..lod_chunk_size {
            let i = z * lod_chunk_size + x as usize; // Index of each vertex in vertex array at (x,z)
            let v = i + z as usize; // v is the index of the top left corner of each square
            let corner:[usize;4] = [
                v,
                v + 1,
                v + lod_chunk_size + 1,
                v + lod_chunk_size + 2];

            // First triangle

            indices_out.push(corner[0]);
            indices_out.push(corner[1]);
            indices_out.push(corner[2]);

            // Second triangle

            indices_out.push(corner[1]);
            indices_out.push(corner[3]);
            indices_out.push(corner[2]);
        }
    }
}
