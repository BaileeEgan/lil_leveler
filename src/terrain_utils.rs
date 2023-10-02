use super::structs::*;
use crate::marching_cubes::*;
use std::collections::HashMap;
use std::collections::HashSet;
use gdnative::api::ArrayMesh;
use gdnative::api::File;
use gdnative::api::MeshDataTool;
use gdnative::prelude::*;
use gdnative::api::Resource;
use gdnative::api::SurfaceTool;
use gdnative::api::MeshInstance;


enum Action {
	TerrainEdit(HashMap<usize,i32>),
	VertexColor(HashMap<usize,Color8>),
}

#[derive(NativeClass)]
#[inherit(Resource)]
pub struct TerrainUtils {
	heights:Vec<i32>,
	map_size:usize,
	num_chunks:usize,
	chunk_size:usize,

	undo_stack:Vec<Action>,
	redo_stack:Vec<Action>,

	vertex_colors:Vec<usize>,
	color_list:Vec<Color8>,

	chunk_vertices:Vec<Vec<Vector3>>,
	chunk_indices:Vec<Vec<usize>>,

	terrain_vertices: Vec<Vector3>,
	terrain_indices: Vec<usize>,
	terrain_vertex_map: HashMap<Vector3Key,usize>,

	shade_smooth:bool,

	edited_positions:HashMap<usize,i32>,
	edited_colors:HashMap<usize,Color8>,

	chunks_at_seam:HashSet<usize>,
	chunks:Vec<Ref<MeshInstance>>,
	is_drawing:bool,

	chunk_lod:Vec<usize>,

	active_chunk_position:(i32,i32),
}

#[methods]
impl TerrainUtils {

    fn new(_owner: &Resource) -> Self {
        TerrainUtils {

			heights:Vec::new(),
			map_size: 0,
			num_chunks: 0,
			chunk_size: 0,

			vertex_colors:Vec::new(),
			color_list:Vec::new(),

			undo_stack:Vec::new(),
			redo_stack:Vec::new(),

			chunk_vertices:Vec::new(),
			chunk_indices:Vec::new(),
			chunk_lod:Vec::new(),
			active_chunk_position:(0,0),

			terrain_vertices:Vec::new(),
			terrain_indices:Vec::new(),
			terrain_vertex_map:HashMap::new(),

			shade_smooth:true,

			edited_positions: HashMap::new(),
			edited_colors: HashMap::new(),
			chunks_at_seam: HashSet::new(),
			chunks: Vec::new(),
			is_drawing: false,
			
        }
    }

	#[export]
	fn init_params (&mut self, _owner:&Resource, _num_chunks:i64, _chunk_size:i64, _heights:TypedArray<i32>, _color_list:TypedArray<Color>, _vertex_colors:TypedArray<i32>) {
		self.num_chunks = _num_chunks.max(1) as usize;
		self.chunk_size = _chunk_size.max(1) as usize;
		self.map_size = self.num_chunks * self.chunk_size;
		self.color_list.clear();
		self.vertex_colors.clear();
		self.chunk_vertices.clear();
		self.chunk_indices.clear();
		for _i in 0..self.num_chunks * self.num_chunks {
			self.chunk_vertices.push(Vec::new());
			self.chunk_indices.push(Vec::new());
		}
		let total_map_size = self.map_size * self.map_size;
		self.heights = vec![0; total_map_size];
		self.vertex_colors = vec![0; total_map_size];
		self.color_list = vec![Color8::new(0,0,0,0); 1];
		if _heights.len() as usize == total_map_size && _vertex_colors.len() as usize == total_map_size {
			for i in 0..total_map_size {
				self.heights[i] = _heights.get(i as i32);
				self.vertex_colors[i] =_vertex_colors.get(i as i32) as usize;
			}
		}
		if _color_list.len() > 0 {
			for i in 1.._color_list.len() {
				self.color_list.push(Color8::from_color(_color_list.get(i)));
			}
		}
		self.chunk_lod = vec![0; self.num_chunks * self.num_chunks];
	}
	

	#[export]
	fn init_chunk_meshes (&mut self, _owner:&Resource, chunk_array:VariantArray) {
		self.chunks.clear();
		for item in chunk_array.iter() {
			let item = item.try_to_object::<MeshInstance>();
			if let Some(item) = item {
				self.chunks.push(item);
			}
		}
		self.generate_all_meshes(_owner);
	}

	fn get_color_index (&self, color:Color8) -> usize {
		let color_list_length = self.color_list.len();
		let mut new_color_index = color_list_length;
		for i in 0..color_list_length {
			if color == self.color_list[i] {
				return i;
			}
		}
		return color_list_length;
	}

	#[export]
	fn update_lod (&mut self, _owner:&Resource, global_x:i32, global_z:i32) {
		let chunk_col = global_x / self.chunk_size as i32;
		let chunk_row = global_z / self.chunk_size as i32;
		
		if self.active_chunk_position.0 != chunk_col || self.active_chunk_position.1 != chunk_row {
			self.active_chunk_position = (chunk_col, chunk_row);
			let mut chunks_to_update:Vec<usize> = Vec::new();
			for i in 0..self.chunk_lod.len() {
				let dist = Vector2::new(chunk_col as f32, chunk_row as f32).distance_to(Vector2::new((i % self.num_chunks) as f32, (i / self.num_chunks) as f32));
				let lod = (dist / 1.0).floor() as usize;
				self.chunk_lod[i] = lod;
				chunks_to_update.push(i);
			}
			if chunks_to_update.len() > 0 {
				for i in chunks_to_update.iter() {
					self.update_chunk(_owner, *i);
					self.generate_chunk_mesh(_owner, *i);
				}
			}
		}
	}

	#[export]
	fn set_chunk_lod (&mut self, _owner:&Resource, chunk_id:usize, lod:usize) {
		self.chunk_lod[chunk_id] = lod;
		self.update_chunk(_owner, chunk_id);
		self.generate_chunk_mesh(_owner, chunk_id);
	}

	fn set_vertex_color (&mut self, x:i32, z:i32, color:Color8) {
		let x = x.max(0).min(self.map_size as i32 - 1) as usize;
		let z = z.max(0).min(self.map_size as i32 - 1) as usize;
		// Check if color exists and grab its color_index
		let color_list_length = self.color_list.len();
		let mut new_color_index = color_list_length;
		for i in 0..color_list_length {
			if color == self.color_list[i] {
				new_color_index = i;
				break;
			}
		}
		// Otherwise add color to color_list
		if new_color_index == color_list_length {
			self.color_list.push(color);
		}
		self.vertex_colors[z * self.map_size + x] = new_color_index;
	}

	#[export]
	fn get_num_chunks (&self, _owner:&Resource) -> i64 { self.num_chunks as i64 }
	#[export]
	fn get_chunk_size (&self, _owner:&Resource) -> i64 { self.chunk_size as i64 }
	

	fn get_vertex_color (&self, x:i32, z:i32) -> Color8 {
		let x = x.max(0).min(self.map_size as i32 - 1) as usize;
		let z = z.max(0).min(self.map_size as i32 - 1) as usize;
		let color_index = self.vertex_colors[z * self.map_size + x];
		return self.color_list[color_index];
	}

	
	

	#[export]
	pub fn clear_heights (&mut self, _owner:&Resource) {
		//for i in 0..self.heights.len() {
		//	self.heights[i] = 0;
		//}
		self.heights = vec![0; self.map_size * self.map_size];
	}

	fn height (&self, global_x:i32, global_z:i32) -> i32 {
		let idx:i32 = global_z * self.map_size as i32 + global_x;
		return self.heights[idx as usize];
	}

	fn height_by_index (&self, index:usize) -> i32 {
		let x = index % self.map_size;
		let z = index / self.map_size;
		return self.height(x as i32, z as i32);
	}

	fn set_height_at (&mut self, global_x:i32, global_z:i32, height:i32) {
		let idx:i32 = global_z * self.map_size as i32 + global_x;
		self.heights[idx as usize] = height;
	}

	fn set_height_by_index (&mut self, index:usize, height:i32) {
		let x = index % self.map_size;
		let z = index / self.map_size;
		self.set_height_at(x as i32,z as i32, height);
	}

	#[export]
	pub fn get_heights(&self, _owner:&Resource) -> TypedArray<i32>{
		let mut arr:TypedArray<i32> = TypedArray::new();
		for i in 0..self.heights.len() {
			arr.insert(i as i32, self.heights[i]);
		}
		return arr;
	}

	#[export]
	pub fn get_vertex_colors(&self, _owner:&Resource) -> TypedArray<i32> {
		let mut arr:TypedArray<i32> = TypedArray::new();
		for i in 0..self.vertex_colors.len() {
			arr.insert(i as i32, self.vertex_colors[i] as i32);
		}
		return arr;
	}

	#[export]
	pub fn get_color_list(&self, _owner:&Resource) -> TypedArray<Color> {
		let mut arr:TypedArray<Color> = TypedArray::new();
		for i in 0..self.color_list.len() {
			arr.insert(i as i32, self.color_list[i].to_color());
		}
		return arr;
	}

	#[export]
	fn get_heights_for_collision (&mut self, _owner:&Resource) -> TypedArray<f32> {
		let mut v:Vec<f32> = Vec::new();
		let map_sized_plus = self.map_size + 1;
		for z in 0..map_sized_plus {
			let z_min = z.min(self.map_size - 1);
			for x in 0..map_sized_plus {
				let x_min = x.min(self.map_size - 1);
				let y = self.height(x_min as i32, z_min as i32);
				v.push(y as f32 + 0.5);
			}
		}
		return TypedArray::from_vec(v);
	}

	
	#[export]
	fn get_height (&self, _owner:&Resource, global_x:i32, global_z:i32) -> i32 {
		let x_clamp = (global_x.max(0) as i32).min(self.map_size as i32 - 1);
		let z_clamp = (global_z.max(0) as i32).min(self.map_size as i32 - 1);
		//let idx:i32 =  z_clamp * self.map_size as i32 + x_clamp;
		return self.height(x_clamp, z_clamp);
	}

	#[export]
	fn get_color (&self, _owner:&Resource, global_x:i32, global_z:i32) -> Color {
		let x_clamp = (global_x.max(0) as i32).min(self.map_size as i32 - 1);
		let z_clamp = (global_z.max(0) as i32).min(self.map_size as i32 - 1);
		let idx:i32 =  z_clamp * self.map_size as i32 + x_clamp;
		return self.color_list[self.vertex_colors[idx as usize]].to_color();
	}

	#[export]
	pub fn set_shade_smooth (&mut self, _owner:&Resource, value:bool) {
		self.shade_smooth = value;
	}

	#[export]
	pub fn in_bounds (&mut self, _owner:&Resource, global_x:i32, global_z:i32) -> bool {
		return global_x > -1 && global_z > -1 && global_x < self.map_size as i32 && global_z < self.map_size as i32;
	}

	#[export]
	pub fn get_vertex_count (&self, _owner:&Resource) -> i32 {
		let mut count:usize = 0;
		for vec in self.chunk_vertices.iter() {
			count += vec.len();
		}
		return count as i32;
	}

	#[export]
	pub fn end_stroke(&mut self, _owner:&Resource) {
		self.is_drawing = false;
		if self.edited_positions.len() > 0 {
			self.undo_stack.push(Action::TerrainEdit(self.edited_positions.clone()));
		}
		for id in self.chunks_at_seam.iter().cloned().collect::<Vec<usize>>() {
			self.update_chunk(_owner, id);
			self.generate_chunk_mesh(_owner, id);
		}
		self.edited_positions.clear();
		self.chunks_at_seam.clear();
	}

	#[export]
	pub fn end_paint_stroke(&mut self, _owner:&Resource) {
		self.is_drawing = false;
		if self.edited_colors.len() > 0 {
			self.undo_stack.push(Action::VertexColor(self.edited_colors.clone()));
		}
		self.edited_colors.clear();
	}

	fn get_grid_positions_in_radius (&self, global_x:i32, global_z:i32, radius:i32) -> Vec<(i32,i32,i32)>{
		let mut positions:Vec<(i32,i32,i32)> = Vec::new();

		let mut add_position = |gx:i32, gz:i32| {
			if gx > -1 && gz > -1 && gx < self.map_size as i32 && gz < self.map_size as i32 {
				positions.push((gx as i32, self.height(gx as i32, gz as i32), gz as i32));
			}
		};

		let radius_squared = 0.25 * (radius as f32).powf(2.0);
		
		for z in 0..radius as i32 {
			let z_squared = (z as f32).powf(2.0);

			for x in 0..radius as i32 {

				let x_squared = (x as f32).powf(2.0);
				if z_squared + x_squared > radius_squared { continue; }
				
				if x == 0 {
					if z == 0 { 
						add_position(global_x, global_z);
					}
					else {
						add_position(global_x, global_z - z);
						add_position(global_x, global_z + z);
					}
				}
				else {
					if z == 0 {
						add_position(global_x - x, global_z);
						add_position(global_x + x, global_z);
					}
					else {
						add_position(global_x - x, global_z - z);
						add_position(global_x + x, global_z - z);
						add_position(global_x - x, global_z + z);
						add_position(global_x + x, global_z + z);
					}
				}
			}
		}
		return positions;
	}


	#[export]
	fn clear_vertex_colors (&mut self, _owner:&Resource) {
		self.vertex_colors = vec![0; self.map_size * self.map_size];
		self.color_list = vec![Color8::new(0,0,0,0); 1]
	}

	/*#[export]
	pub fn paint_face (&mut self, _owner:&Resource, position:Vector3, brush_radius:f32, color:Color, opacity:f32, blend_mode:i32) {
		if !self.is_drawing {
			self.is_drawing = true;
		}

		let color8 = Color8::from_color(color);

		let mut chunks_to_update:HashSet<usize> = HashSet::new();

		let radius = brush_radius as i32;

		for z in -radius..(radius + 1) {
			let global_z = position.z as i32 + z;
			if global_z < 0 || global_z >= self.map_size as i32 { continue; }
			let chunk_row = global_z as usize / self.chunk_size;
			for x in -radius..(radius + 1) {
				let global_x = position.x as i32 + x;
				if global_x < 0 || global_x >= self.map_size as i32 { continue; }
				let chunk_col = global_x as usize / self.chunk_size;
				let chunk_id = chunk_row * self.num_chunks + chunk_col;
				chunks_to_update.insert(chunk_id);
			}
		}

		let radius_squared = brush_radius * brush_radius * 0.25;
		let mdt = MeshDataTool::new();

		for chunk_id in chunks_to_update.iter() {
			let mesh_instance = self.chunks[*chunk_id];
			let mesh_instance = unsafe { mesh_instance.assume_safe() };
			let mesh = mesh_instance.mesh();
			if let Some(mesh) = mesh {
				let mesh = mesh.cast::<ArrayMesh>().unwrap();
				let mesh = unsafe { mesh.assume_safe() };
				mdt.create_from_surface(mesh.clone(), 0).unwrap();


				let offset_x = self.chunk_size as i32 * (*chunk_id as i32 % self.num_chunks as i32);
				let offset_z = self.chunk_size as i32 * (*chunk_id as i32 / self.num_chunks as i32);
				
				for i in 0..mdt.get_face_count() {

					let vert_indices = [
						mdt.get_face_vertex(i, 0),
						mdt.get_face_vertex(i, 1),
						mdt.get_face_vertex(i, 2),
					];

					let vert_points = [
						mdt.get_vertex(vert_indices[0]),
						mdt.get_vertex(vert_indices[1]),
						mdt.get_vertex(vert_indices[2])
					];

					let centroid = centroid(vert_points);

					let local_vert = mdt.get_vertex(i);
					let mut global_vert = local_vert + Vector3::new(offset_x as f32, 0.0, offset_z as f32);
		
					let diff:Vector3 = global_vert - centroid;
		
					if diff.square_length() <= radius_squared {
						global_vert.y /= self.step_height;
						let vert_key = Vector3Key::from_vector3(global_vert);

						if !self.edited_colors.contains_key(&vert_key) {
							let mut current_color = Color8::new(0, 0, 0, 0);
							let saved_color = self.vertex_colors.get(&vert_key);
							if let Some(saved_color) = saved_color {
								current_color = *saved_color;
							}
	
							let new_color:Color8 = match blend_mode {
								0 => current_color.mix(color8, opacity),
								1 => current_color.add(color8, opacity),
								2 => current_color.multiply(color8, opacity),
								3 => current_color.subtract(color8, opacity),
								_ => current_color.mix(color8, opacity)
							};
	
							self.vertex_colors.insert(vert_key, new_color);
							self.edited_colors.insert(vert_key, new_color);
							mdt.set_vertex_color(i, new_color.to_color());
						}
						else {
							mdt.set_vertex_color(i, self.edited_colors.get(&vert_key).unwrap().to_color());
						}
					}
				}
				mesh.surface_remove(0);
				mdt.commit_to_surface(mesh).unwrap();
			}
		}
	}*/

	#[export]
	pub fn paint_vertex (&mut self, _owner:&Resource, position:Vector3, brush_radius:f32, color:Color, opacity:f32, blend_mode:i32) {
		if !self.is_drawing {
			self.is_drawing = true;
			self.edited_positions.clear();
			self.redo_stack.clear();
		}

		let color8 = Color8::from_color(color);

		let mut chunks_to_update:HashSet<usize> = HashSet::new();

		let radius = brush_radius as i32;
		let radius_squared = brush_radius * brush_radius * 0.25;

		for z in -radius..(radius + 1) {
			let global_z = position.z as i32 + z;
			if global_z < 0 || global_z >= self.map_size as i32 { continue; }
			let chunk_row = global_z as usize / self.chunk_size;
			for x in -radius..(radius + 1) {
				let global_x = position.x as i32 + x;
				if global_x < 0 || global_x >= self.map_size as i32 { continue; }
				let chunk_col = global_x as usize / self.chunk_size;
				let chunk_id = chunk_row * self.num_chunks + chunk_col;
				chunks_to_update.insert(chunk_id);
			}
		}

		//let mdt = MeshDataTool::new();

		/*for chunk_id in chunks_to_update.iter() {
			for i in 0..self.chunk_vertices[*chunk_id].len() {
				let vert = self.chunk_vertices[*chunk_id][i];
				let offset_x = self.chunk_size as i32 * (*chunk_id as i32 % self.num_chunks as i32);
				let offset_z = self.chunk_size as i32 * (*chunk_id as i32 / self.num_chunks as i32);
				let mut global_vert = vert + Vector3::new(offset_x as f32, 0.0, offset_z as f32);

				let diff:Vector3 = Vector3::new(global_vert.x - position.x, global_vert.y - position.y, global_vert.z - position.z);

				if diff.square_length() <= radius_squared {
					global_vert.y /= self.step_height;
					let vert_key = Vector3Key::from_vector3(global_vert);
					if !self.edited_colors.contains_key(&vert_key) {
						let current_color = self.get_vertex_color(vert_key);
						let new_color:Color8 = current_color.blend(color8, opacity, blend_mode); 
						self.set_vertex_color(vert_key, new_color);
						self.edited_colors.insert(vert_key, current_color);
					}
				}
			}
		}

		// Update the chunks
		for id in chunks_to_update.iter() {
			self.generate_chunk_mesh(_owner, *id);
		}*/



		let mdt = MeshDataTool::new();

		for chunk_id in chunks_to_update.iter() {

			//self.chunks_at_seam.remove(&(*chunk_id as usize));

			let mesh_instance = self.chunks[*chunk_id];
			let mesh_instance = unsafe { mesh_instance.assume_safe() };
			let mesh = mesh_instance.mesh();
			if let Some(mesh) = mesh {
				let mesh = mesh.cast::<ArrayMesh>().unwrap();
				let mesh = unsafe { mesh.assume_safe() };
				mdt.create_from_surface(mesh.clone(), 0).unwrap();

				let offset_x = self.chunk_size as i32 * (*chunk_id as i32 % self.num_chunks as i32);
				let offset_z = self.chunk_size as i32 * (*chunk_id as i32 / self.num_chunks as i32);
				
				for i in 0..mdt.get_vertex_count() {
					let local_vert = mdt.get_vertex(i);
					let local_vert = Vector2::new(local_vert.x, local_vert.z);
					let global_vert = local_vert + Vector2::new(offset_x as f32, offset_z as f32);
		
					let diff:Vector2 = Vector2::new(global_vert.x - position.x,  global_vert.y - position.z);
		
					if diff.square_length() <= radius_squared {
						let x = (global_vert.x.floor() as i32).max(0).min(self.map_size as i32 - 1) as usize;
						let z = (global_vert.y.floor() as i32).max(0).min(self.map_size as i32 - 1) as usize;
						let index = z * self.map_size + x;

						if !self.edited_colors.contains_key(&index) {
							let current_color = self.color_list[self.vertex_colors[index]];
							let new_color:Color8 = current_color.blend(color8, opacity, blend_mode); 
							self.set_vertex_color(x as i32, z as i32, new_color);
							self.edited_colors.insert(index, current_color);
							mdt.set_vertex_color(i, new_color.to_color());
						}
						else {
							mdt.set_vertex_color(i, self.color_list[self.vertex_colors[index]].to_color());
						}
					}
				}
				mesh.surface_remove(0);
				mdt.commit_to_surface(mesh).unwrap();
			}
		}
	}

	/*pub fn color_by_slope (&mut self, _owner:&Resource, min_slope:f32, max_slope:f32, color:Color, strength:f32, blend_mode:i32) {
		let color8 = Color8::from_color(color);
		let mut changed_verts:HashSet<Vector2Key> = HashSet::new();
		for chunk_id in 0..self.chunk_normals.len() {

			let offset_x = self.chunk_size as i32 * (chunk_id as i32 % self.num_chunks as i32);
			let offset_z = self.chunk_size as i32 * (chunk_id as i32 / self.num_chunks as i32);

			for i in 0..self.chunk_normals[chunk_id].len() {
				let normal = self.chunk_normals[chunk_id][i];
				let slope = 1.0 - normal.dot(Vector3::new(0.0, 1.0, 0.0));
				if slope >= min_slope && slope <= max_slope {
					let local_vert = self.chunk_vertices[chunk_id][i];
					let global_vert = local_vert + Vector3::new(offset_x as f32, 0.0, offset_z as f32);
					let global_vert_key = Vector2Key::from_vector3(global_vert);
					if !changed_verts.contains(&global_vert_key) {
						let current_color = self.get_vertex_color(global_vert_key);
						let new_color:Color8 = current_color.blend(color8, strength, blend_mode); 
						self.set_vertex_color(global_vert_key, new_color);
						changed_verts.insert(global_vert_key);
					}
				}
			}
		}
		self.generate_all_meshes(_owner);
	}

	pub fn replace_color (&mut self, _owner:&Resource, old_color:Color, similarity:f32, new_color:Color, strength:f32, blend_mode:i32) {
		let old_color = Color8::from_color(old_color);
		let new_color = Color8::from_color(new_color);
		let mut changed_verts:HashMap<Vector2Key,Color8> = HashMap::new();
		let threshold = similarity * similarity;
		for (vert_key, color_index) in self.vertex_colors.iter() {
			let color = self.color_list[*color_index];
			if !changed_verts.contains_key(vert_key) {
				let sq_similarity:f32 = color.squared_similarity(old_color);
				if sq_similarity >= threshold {
					let new_color:Color8 = color.blend(new_color, strength, blend_mode);
					changed_verts.insert(*vert_key, new_color);
				}
			}
		}
		for (vert_key, color) in changed_verts.iter() {
			self.set_vertex_color(*vert_key, *color);
		}
		self.generate_all_meshes(_owner);
	}*/

	#[export]
	pub fn draw_at (&mut self,_owner:&Resource, global_x:i32, global_z:i32, brush_size:Vector2, brush_mode:i32, button_index:i32, is_height_locked:bool, locked_height:i32) -> bool {
		if !self.is_drawing {
			self.is_drawing = true;
			self.edited_positions.clear();
			self.redo_stack.clear();
		}

		let mut chunks_to_update:HashSet<i32> = HashSet::new();
		let brush_height = brush_size.y as i32;

		if brush_mode == 5 && self.in_bounds(_owner, global_x, global_z)  {
			let positions_in_range:Vec<(i32,i32,i32)> = self.get_grid_positions_in_radius(global_x, global_z, brush_size.x as i32 + 1);
			let num_positions_in_range:usize = positions_in_range.len();
			let mut target_height:f32 = 0.0;
			
			if is_height_locked {
				target_height = self.height(global_x as i32, global_z as i32) as f32;
			}
			else {
				for (x,y,z) in positions_in_range.iter() {
					target_height += *y as f32 / (num_positions_in_range as f32)
				}
			}

			let mut squares_average:f32 = 0.0;

			for (x,y,z) in positions_in_range.iter() {
				let diff_sq:f32 = (*y as f32 - target_height).powf(2.0) ;
				squares_average += diff_sq / (num_positions_in_range as f32);
			}

			if squares_average > 0.0 {
				for (x,y,z) in positions_in_range.iter() {
					if *y != target_height.round() as i32 && (*z as f32 - global_z as f32).powf(2.0) + (*x as f32 - global_x as f32).powf(2.0) <= 0.25 * brush_size.x.powf(2.0){
						let sign:i32 = if *y < target_height.round() as i32 { -1 } else { 1 };

						let new_height = *y - sign;
						if *y != new_height {
							let index = *z as usize * self.map_size + *x as usize;
							if !self.edited_positions.contains_key(&index) {
								self.edited_positions.insert(index, *y);
							}
						
							self.set_height_at(*x, *z, new_height);
							//self.heights[index as usize] = new_height;
							let chunk_id:i32 = ((z / self.chunk_size as i32) * self.num_chunks as i32 + (x / self.chunk_size as i32)) as i32;
							self.check_seam_condition(*x as i32, *z as i32);  
							chunks_to_update.insert(chunk_id);
						}
					}
				}
			}
		}
		else if brush_mode != 5 {
			let positions_in_range:Vec<(i32,i32,i32)> = self.get_grid_positions_in_radius(global_x, global_z, brush_size.x as i32);
			for (x,y,z) in positions_in_range.iter() {
				let chunk_id = self.draw_height_at(*x as i32, *z as i32, brush_mode, brush_height, button_index, is_height_locked, locked_height);
				chunks_to_update.insert(chunk_id);
			}
		}


		let mut do_update_mesh:bool = false;
		for chunk_id in chunks_to_update.iter() {
			if *chunk_id > -1 {
				self.update_chunk(_owner, *chunk_id as usize);
				self.generate_chunk_mesh(_owner, *chunk_id as usize);
				self.chunks_at_seam.remove(&(*chunk_id as usize));
				do_update_mesh = true;
			}
		}

		return do_update_mesh;
	}

	fn draw_height_at (&mut self, global_x:i32, global_z:i32, draw_mode:i32, brush_height:i32, button_index:i32, is_height_locked:bool, locked_height:i32) -> i32 {
		if global_x < 0 || global_z < 0 || global_x >= self.map_size as i32 || global_z >= self.map_size as i32 {
			return -1;
		}

		let i:usize = global_z as usize * self.map_size + global_x as usize;
		if self.edited_positions.contains_key(&i) {
			return -1;
		}

		let current_height = self.height(global_x as i32, global_z as i32);
		let mut new_height = current_height;

		if (draw_mode == 0 || draw_mode == 1) &&
			((current_height == locked_height && is_height_locked) || !is_height_locked) {
				if (draw_mode == 0 && button_index == 1) || (draw_mode == 1 && button_index == 2) {
					new_height = current_height + brush_height as i32;
				}
				else if (draw_mode == 1 && button_index == 1) || (draw_mode == 0 && button_index == 2) {
					new_height = (current_height - brush_height as i32).max(0);
				}
		}
		else if draw_mode == 2 {
			if is_height_locked && current_height == locked_height {
				new_height = current_height + brush_height as i32;
			}
			else if !is_height_locked {
				new_height = brush_height as i32;
			}
		}

		else if draw_mode == 3 {
			if is_height_locked && current_height < locked_height + brush_height as i32 {
				new_height = locked_height + brush_height as i32;
			}
			else if !is_height_locked && current_height < brush_height as i32 {
				new_height = brush_height as i32;
			}
		}
		else if draw_mode == 4 {
			if is_height_locked && current_height > locked_height + brush_height as i32 {
				new_height = locked_height + brush_height as i32;
			}
			else if !is_height_locked && current_height > brush_height as i32 {
				new_height = brush_height as i32;
			}
		}

		if new_height != current_height {
			self.edited_positions.insert(i, current_height);
			self.set_height_at(global_x as i32, global_z as i32, new_height);

			let chunk_col = global_x as i32 / self.chunk_size as i32;
			let chunk_row = global_z as i32 / self.chunk_size as i32;

			self.check_seam_condition(global_x, global_z);
			
			return (chunk_row * self.num_chunks as i32 + chunk_col) as i32;
		}

		return -1;
	}
	
	fn check_seam_condition (&mut self, global_x:i32, global_z:i32) {
		let global_x = global_x as usize;
		let global_z = global_z as usize;

		let chunk_size:usize = self.chunk_size;
		let num_chunks = self.num_chunks;

		let chunk_col = global_x as usize / chunk_size;
		let chunk_row = global_z as usize / chunk_size;

		let x_border_min = global_x % chunk_size == 0 && chunk_col > 0;
		let x_border_max = global_x % chunk_size == 1 && chunk_col < num_chunks - 1;

		let z_border_min = global_z % chunk_size == 0 && chunk_row > 0;
		let z_border_max = global_z % chunk_size == 1  && chunk_row < num_chunks - 1;


		if x_border_min {
			if z_border_min {
				self.chunks_at_seam.insert((chunk_row - 1) * num_chunks + chunk_col - 1);
				self.chunks_at_seam.insert(chunk_row * num_chunks  + chunk_col - 1);
				self.chunks_at_seam.insert((chunk_row - 1) * num_chunks  + chunk_col);
			}
			else if z_border_max {
				self.chunks_at_seam.insert((chunk_row + 1) * num_chunks + chunk_col - 1);
				self.chunks_at_seam.insert(chunk_row * num_chunks + chunk_col - 1);
				self.chunks_at_seam.insert((chunk_row + 1) * num_chunks + chunk_col);
			}
			else {
				self.chunks_at_seam.insert(chunk_row * num_chunks + chunk_col - 1);
			}
		}
		else if x_border_max {
			if z_border_min {
				self.chunks_at_seam.insert((chunk_row - 1) * num_chunks + chunk_col + 1);
				self.chunks_at_seam.insert(chunk_row * num_chunks + chunk_col + 1);
				self.chunks_at_seam.insert((chunk_row - 1) * num_chunks + chunk_col);
			}
			else if z_border_max {
				self.chunks_at_seam.insert((chunk_row + 1) * num_chunks + chunk_col + 1);
				self.chunks_at_seam.insert((chunk_row + 1) * num_chunks + chunk_col);
				self.chunks_at_seam.insert(chunk_row * num_chunks + chunk_col + 1);
			}
			else {
				self.chunks_at_seam.insert(chunk_row * num_chunks + chunk_col + 1);
			}
		}
		else {
			if z_border_min {
				self.chunks_at_seam.insert((chunk_row - 1) * num_chunks + chunk_col);
			}
			else if z_border_max {
				self.chunks_at_seam.insert((chunk_row + 1) * num_chunks + chunk_col);
			}
		}
	}

	#[export]
	fn resize_terrain (&mut self, _owner:&Resource, new_chunk_size:i32, new_num_chunks:i32, x_move:i32, z_move:i32) {
		let new_map_size:i32 = new_chunk_size * new_num_chunks;
		let diff:i32 = new_map_size as i32 - self.map_size as i32;
		if diff == 0 { return; }

		let half_size:i32 = ((diff as f32) / 2.0).trunc() as i32;

		let offset_x:i32 = match x_move {
			0 => 0,
			1 => half_size,
			2 => diff,
			_ => 0
		};
		let offset_z:i32 = match z_move {
			0 => 0,
			1 => half_size,
			2 => diff,
			_ => 0
		};

		let mut new_heights:Vec<i32> = vec![0; (new_map_size * new_map_size) as usize];
		let mut new_colors:Vec<usize> =  vec![0; (new_map_size * new_map_size) as usize];
		

		if diff > 0 {
			for z in 0..self.map_size {
				for x in 0..self.map_size {
					let idx = z * self.map_size + x;
					let current_height = self.heights[idx];
					let current_color = self.vertex_colors[idx];
					let new_idx = (z as i32 + offset_z) * new_map_size as i32 + x as i32 + offset_x as i32;
					new_heights[new_idx as usize] = current_height;
					new_colors[new_idx as usize] = current_color;
				}
			}
		}
		else {
			for z in 0..new_map_size {
				for x in 0..new_map_size {
					let idx = (z as i32 - offset_z) * self.map_size as i32 + (x as i32 - offset_x);
					let current_height = self.heights[idx as usize];
					let current_color = self.vertex_colors[idx as usize];
					let new_idx = z * new_map_size + x;
					new_heights[new_idx as usize] = current_height;
					new_colors[new_idx as usize] = current_color;
				}
			}
		}


		self.heights = new_heights;
		self.vertex_colors = new_colors;

		self.chunk_size = new_chunk_size as usize;
		self.num_chunks = new_num_chunks as usize;
		self.map_size = new_map_size as usize;


		self.chunk_vertices.clear();
		self.chunk_indices.clear();
		self.chunk_lod = vec![0; self.num_chunks * self.num_chunks];
		for _chunk_id in 0..self.num_chunks * self.num_chunks {
			self.chunk_vertices.push(Vec::new());
			self.chunk_indices.push(Vec::new());
		}

		self.update_all_chunks(_owner);
		self.update_terrain_arrays();
		self.edited_positions.clear();
		self.chunks_at_seam.clear();
	}



	#[export]
	fn update_terrain (&mut self, _owner:&Resource) {
		self.update_all_chunks(_owner);
		self.update_terrain_arrays();
	}

	#[export]
	fn update_terrain_arrays_for_chunks (&mut self, _owner:&Resource, chunk_ids:TypedArray<i32>) {
		// Index vertices
		self.terrain_vertex_map.clear();
		self.terrain_vertices.clear();
		self.terrain_indices.clear();

		for c in 0..chunk_ids.len() {
			let chunk_id = chunk_ids.get(c) as usize;
			let offset_x = self.chunk_size as i32 * (chunk_id as i32 % self.num_chunks as i32);
			let offset_z = self.chunk_size as i32 * (chunk_id as i32 / self.num_chunks as i32);
			
			for i in (0..self.chunk_indices[chunk_id].len()).step_by(3) {
				let mut face_verts:[Vector3; 3] = [Vector3::new(0.0, 0.0, 0.0); 3];
				

				for n in 0..3 {
					let idx = self.chunk_indices[chunk_id][i + n];
					let vert:Vector3 = self.chunk_vertices[chunk_id][idx as usize] + Vector3::new(offset_x as f32, 0.0, offset_z as f32);
					let vert_key:Vector3Key = Vector3Key::new(vert.x, vert.y, vert.z);
					let vert_index = self.terrain_vertex_map.get(&vert_key);
					if let Some(vert_index) = vert_index {
						self.terrain_indices.push(*vert_index);
					}
					else {
						let new_index = self.terrain_vertices.len();
						self.terrain_vertex_map.insert(vert_key, new_index);
						
						self.terrain_vertices.push(vert);
						self.terrain_indices.push(new_index);
					}
				}
			}
		}
	}

	fn update_terrain_arrays (&mut self) {
		// Index vertices
		self.terrain_vertex_map.clear();
		self.terrain_vertices.clear();
		self.terrain_indices.clear();

		for chunk_id in 0..self.chunk_indices.len() {
			let offset_x = self.chunk_size as i32 * (chunk_id as i32 % self.num_chunks as i32);
			let offset_z = self.chunk_size as i32 * (chunk_id as i32 / self.num_chunks as i32);
			
			for i in (0..self.chunk_indices[chunk_id].len()).step_by(3) {
				let mut face_verts:[Vector3; 3] = [Vector3::new(0.0, 0.0, 0.0); 3];

				for n in 0..3 {
					let idx = self.chunk_indices[chunk_id][i + n];
					let vert:Vector3 = self.chunk_vertices[chunk_id][idx as usize] + Vector3::new(offset_x as f32, 0.0, offset_z as f32);
					let vert_key:Vector3Key = Vector3Key::new(vert.x, vert.y, vert.z);
					let vert_index = self.terrain_vertex_map.get(&vert_key);
					if let Some(vert_index) = vert_index {
						self.terrain_indices.push(*vert_index);
					}
					else {
						let new_index = self.terrain_vertices.len();
						self.terrain_vertex_map.insert(vert_key, new_index);
						
						self.terrain_vertices.push(vert);
						self.terrain_indices.push(new_index);
					}
				}
			}
		}
	}


	//indices_out:&mut Vec<usize>, vertices_out:&mut Vec<Vector3>, normals_out:&mut Vec<Vector3>, 
	#[export]
	fn update_chunk (&mut self, _owner:&Resource, chunk_id:usize) {
		self.chunk_vertices[chunk_id].clear();
		self.chunk_indices[chunk_id].clear();

		if self.chunk_lod[chunk_id] > 0 {
			generate_chunk_lod_mesh(&self.heights, chunk_id, self.chunk_size, self.num_chunks, &self.chunk_lod, &mut self.chunk_indices[chunk_id], &mut self.chunk_vertices[chunk_id]);
		}

		else {
			let vertices:Vec<Vector3> = generate_chunk_mesh_from_heightmap(&self.heights, chunk_id, self.chunk_size, self.num_chunks, &self.chunk_lod);
			let mut vertex_map:HashMap<Vector3Key,usize> = HashMap::new();

			for i in (0..vertices.len()).step_by(3) {
				let points = [
					vertices[i],
					vertices[i + 1],
					vertices[i + 2],
				];
				for point in points.iter() {
					let point = *point;
					let vert_key = Vector3Key::from_vector3(point);
					let vert_index = vertex_map.get(&vert_key);
					if let Some(vert_index) = vert_index {
						self.chunk_indices[chunk_id].push(*vert_index);
					}
					else {
						let new_index = self.chunk_vertices[chunk_id].len();
						vertex_map.insert(vert_key, new_index);
						self.chunk_vertices[chunk_id].push(point);
						self.chunk_indices[chunk_id].push(new_index);
					}
				}
			}

		}
	}

	#[export]
	fn update_all_chunks (&mut self, _owner:&Resource) {
		for i in 0..self.chunk_indices.len() {
			self.update_chunk(_owner, i as usize);
		}
	}

	#[export]
    fn generate_chunk_mesh (&mut self, _owner:&Resource, chunk_id:usize) {
		if chunk_id < self.chunks.len() {
			let offset_x = self.chunk_size as i32 * (chunk_id as i32 % self.num_chunks as i32);
			let offset_z = self.chunk_size as i32 * (chunk_id as i32 / self.num_chunks as i32);
			let st = SurfaceTool::new();
			st.begin(4);

			if self.shade_smooth {
				st.add_smooth_group(true);
			}

			for i in 0..self.chunk_vertices[chunk_id].len() {
				let vert = self.chunk_vertices[chunk_id][i];
				let vert_floor:Vector2 = Vector2::new(vert.x.floor() + offset_x as f32, vert.z.floor() + offset_z as f32);
				let color = self.get_vertex_color(vert_floor.x as i32, vert_floor.y as i32);
				st.add_color(color.to_color());
				st.add_vertex(vert);
			}

			for index in self.chunk_indices[chunk_id].iter() {
				st.add_index(*index as i64);
			}

			st.generate_normals(false);

			let mesh = st.commit(GodotObject::null(), 2194432);
			let mesh_instance = unsafe { self.chunks[chunk_id].assume_safe() };
			if let Some(mesh) = mesh {
				mesh_instance.set_mesh(mesh);
			}
		}
    }

	#[export]
	fn generate_all_meshes (&mut self, _owner:&Resource) {
		for i in 0..self.chunks.len() {
			self.generate_chunk_mesh(_owner, i);
		}
	}

	#[export]
	pub fn can_undo (&self, _owner:&Resource) -> bool {
		return self.undo_stack.len() > 0;
	}
	#[export]
	pub fn can_redo (&self, _owner:&Resource) -> bool {
		return self.redo_stack.len() > 0;
	}

	fn do_action (&mut self, owner:&Resource, action:Action) -> Action {
		let mut inv_action:Action;
		match action {
			Action::TerrainEdit(data) => {
				let mut inv_positions:HashMap<usize,i32> = HashMap::new();
				let mut chunks_to_update:HashSet<usize> = HashSet::new();
				
				self.chunks_at_seam.clear();

				// Iterate through saved heights and update the chunks
				for (index,height) in data.iter() {
					let current_height:i32 = self.height_by_index(*index);
					inv_positions.insert(*index, current_height);

					let x = index % self.map_size;
					let z = index / self.map_size;
					let chunk_id:usize = (z / self.chunk_size) * self.num_chunks + (x / self.chunk_size);
					self.set_height_by_index(*index, *height);
					self.check_seam_condition(x as i32, z as i32);  
					chunks_to_update.insert(chunk_id);
				}

				// Update the chunks
				for id in chunks_to_update.iter() {
					self.update_chunk(owner, *id);
					self.generate_chunk_mesh(owner, *id);
				}

				// Update the seams
				for id in self.chunks_at_seam.iter().cloned().collect::<Vec<usize>>() {
					if chunks_to_update.contains(&id) {
						self.update_chunk(owner, id);
						self.generate_chunk_mesh(owner, id);
					}
				}
				self.chunks_at_seam.clear();

				inv_action = Action::TerrainEdit(inv_positions);
			}

			Action::VertexColor(data) => {
				let mut inv_colors:HashMap<usize,Color8> = HashMap::new();
				let mut chunks_to_update:HashSet<usize> = HashSet::new();

				self.chunks_at_seam.clear();

				for (idx,color) in data.iter () {
					let prev_color = self.color_list[self.vertex_colors[*idx]];
					inv_colors.insert(*idx,prev_color);

					let global_z = idx / self.map_size;
					let global_x = idx % self.map_size;
					let chunk_row = global_z as usize / self.chunk_size;
					let chunk_col = global_x as usize / self.chunk_size;
					let chunk_id = chunk_row * self.num_chunks + chunk_col;
					chunks_to_update.insert(chunk_id);
					self.set_vertex_color(global_x as i32, global_z as i32, *color);
					self.check_seam_condition(global_x as i32, global_z as i32);  

				}

				// Update the chunks
				for id in chunks_to_update.iter().cloned().collect::<Vec<usize>>() {
					self.update_chunk(owner, id as usize);
					self.generate_chunk_mesh(owner, id as usize);
				}

				// Update the seams
				for id in self.chunks_at_seam.iter().cloned().collect::<Vec<usize>>() {
					if chunks_to_update.contains(&id) {
						self.update_chunk(owner, id);
						self.generate_chunk_mesh(owner, id);
					}
				}
				inv_action = Action::VertexColor(inv_colors);
			}
		}
		return inv_action;
	}

	#[export]
	pub fn do_undo (&mut self, _owner:&Resource) -> bool {
		let action = self.undo_stack.pop();
		if let Some(action) = action {
			let new_redo:Action = self.do_action(_owner, action);
			self.redo_stack.push(new_redo);
		}
		return self.undo_stack.len() > 0;
	}

	#[export]
	pub fn do_redo (&mut self, _owner:&Resource) -> bool {
		let action = self.redo_stack.pop();
		if let Some(action) = action {
			let new_undo:Action = self.do_action(_owner, action);
			self.undo_stack.push(new_undo);
		}
		return self.redo_stack.len() > 0;
	}

	/*#[export]
	fn data_to_file (&self, _owner:&Resource, path:GodotString, settings:Dictionary) {
		godot_print!("{} {}", self.get_heights(_owner).len(), self.heights.len());
		let file:Ref<File,Unique> = File::new();
		file.open(path, File::WRITE).unwrap();
		let variables= Dictionary::new();
		variables.insert("heights".to_variant(), self.get_heights(_owner));
		variables.insert("num_chunks".to_variant(), self.num_chunks as i64);
		variables.insert("chunk_size".to_variant(), self.chunk_size as i64);
		variables.insert("step_height".to_variant(), self.step_height);
		variables.insert("vertex_colors".to_variant(), self.get_vertex_colors(_owner));
		variables.insert("color_list".to_variant(), self.get_color_list(_owner));
		variables.insert("settings".to_variant(), settings);
		file.store_var(variables, true);
		file.close();
	}

	
	#[export]
	fn init_from_file (&mut self, _owner:&Resource, path:GodotString) {
		let file:Ref<File,Unique> = File::new();
		file.open(path, File::READ).unwrap();
		let variables = file.get_var(true).to_dictionary();
		if variables.contains("heights".to_variant()) {
			self.num_chunks = variables.get("num_chunks").to_i64() as usize;
			self.chunk_size = variables.get("chunk_size").to_i64() as usize;
			self.map_size = self.num_chunks * self.chunk_size;

			self.heights = vec![0; self.map_size * self.map_size];

			self.step_height = variables.get("step_height").to_f64() as f32;

			let _heights = &variables.get("heights").to_int32_array();
			godot_print!("{} {}", self.heights.len(), _heights.len());
			for i in 0.._heights.len() {
				self.heights[i as usize] = _heights.get(i);
			}

			//godot_print!("Leaves: {} | Total: {}", self.height_quadtree.num_leaf_nodes(), self.height_quadtree.len());

			self.vertex_colors = vec![0; self.map_size * self.map_size];
			self.color_list = vec![Color8::new(0,0,0,0); 1];
			self.chunk_lod = vec![0; self.num_chunks * self.num_chunks];
			

			self.chunk_vertices.clear();
			self.chunk_indices.clear();
			self.chunk_normals.clear();
			for _i in 0..self.num_chunks * self.num_chunks {
				self.chunk_vertices.push(Vec::new());
				self.chunk_indices.push(Vec::new());
				self.chunk_normals.push(Vec::new());
			}
		}
		file.close();
	}*/

	#[export]
	fn data_to_obj (&mut self, _owner:&Resource, path:GodotString) {
		let file:Ref<File,Unique> = File::new();
		file.open(path, File::WRITE).unwrap();
		file.store_line("o terrain");
		for vertex in self.terrain_vertices.iter() {
			file.store_line(format!("v {} {} {}", vertex.x, vertex.y, vertex.z));
		}
		file.store_line("s 1");
		for i in (0..self.terrain_indices.len()).step_by(3) {
			file.store_line(format!("f {} {} {}", self.terrain_indices[i + 2] + 1, self.terrain_indices[i + 1] + 1, self.terrain_indices[i] + 1));
		}
		file.close();
	}


	#[export]
	pub fn set_array_mesh (&mut self, _owner:&Resource, mesh_instance:Variant) {
		let mesh_instance = mesh_instance.try_to_object::<MeshInstance>();
		if let Some(mesh_instance) = mesh_instance {
			let mesh_instance = unsafe { mesh_instance.assume_safe() };
			
			let st = SurfaceTool::new();
			st.begin(4);
			if self.shade_smooth {
				st.add_smooth_group(true);
			}

			for i in 0..self.terrain_vertices.len() {
				let vert = self.terrain_vertices[i];
				let color:Color = self.get_vertex_color(vert.x.floor() as i32, vert.z.floor() as i32).to_color();
				st.add_color(color);
				st.add_vertex(vert);
			}

			for index in self.terrain_indices.iter() { st.add_index(*index as i64); }

			st.generate_normals(false);
			
			let mesh = st.commit(GodotObject::null(), 2194432);
			if let Some(mesh) = mesh { mesh_instance.set_mesh(mesh); }
		}
	}
}
