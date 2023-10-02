
use gdnative::api::File;
use gdnative::prelude::*;
use gdnative::api::StaticBody;
use gdnative::api::CollisionShape;
use gdnative::api::HeightMapShape;
use gdnative::api::SurfaceTool;
use gdnative::api::MeshInstance;
use crate::marching_cubes::generate_chunk_mesh;
use crate::marching_cubes::Vector3Key;
use std::collections::HashMap;
use gdnative::nativescript::property::*;
use std::collections::HashSet;

#[derive(NativeClass)]
#[inherit(StaticBody)]
#[register_with(Self::register_properties)]
pub struct TerrainNode {
	step_height:f32,
	heights:Vec<i32>,
	map_size:usize,
	#[property(default=4)]
	num_chunks:i64,
	#[property(default=16)]
	chunk_size:i64,
	locked_height:i32,
	is_height_locked:bool,

	undo_stack:Vec<HashMap<usize,i32>>,
	redo_stack:Vec<HashMap<usize,i32>>,

	chunk_vertices:Vec<Vec<Vector3>>,
	chunk_indices:Vec<Vec<usize>>,

	edited_positions:HashMap<usize,i32>,
	chunks_at_seam:HashSet<usize>,
	chunks:Vec<Ref<MeshInstance>>,
	is_drawing:bool
}

#[methods]
impl TerrainNode {

	fn register_properties(builder: &ClassBuilder<TerrainNode>) {
		builder
            .add_property::<i64>("num_chunks")
            .with_getter(move |my_node: &TerrainNode, _owner: TRef<StaticBody>| my_node.num_chunks)
            .with_setter(move |my_node: &mut TerrainNode, _owner: TRef<StaticBody>, new_value| my_node.num_chunks = new_value)
            .with_default(4)
            .with_hint(IntHint::Range(RangeHint::new(1, 64).with_step(1)))
            .done();
		builder
            .add_property::<i64>("chunk_size")
            .with_getter(move |my_node: &TerrainNode, _owner: TRef<StaticBody>| my_node.chunk_size)
            .with_setter(move |my_node: &mut TerrainNode, _owner: TRef<StaticBody>, new_value| my_node.chunk_size = new_value)
            .with_default(16)
            .with_hint(IntHint::Range(RangeHint::new(4, 64).with_step(1)))
            .done();
	}

    fn new(_owner: TRef<StaticBody>) -> Self {
        TerrainNode {
			step_height: 1.0,
			heights: Vec::new(),
			map_size: 0,
			num_chunks: 4,
			chunk_size: 16,
			locked_height: -1,
			is_height_locked:false,

			undo_stack:Vec::new(),
			redo_stack:Vec::new(),

			chunk_vertices:Vec::new(),
			chunk_indices:Vec::new(),
			edited_positions: HashMap::new(),
			chunks_at_seam: HashSet::new(),
			chunks: Vec::new(),
			is_drawing: false
        }
    }


	#[export]
	fn _ready (&mut self, _owner:TRef<StaticBody>) {
		self.init_params(_owner, self.num_chunks, self.chunk_size, self.step_height);
		self.init_chunks(_owner);
	}

	#[export]
	fn init_params (&mut self, _owner:TRef<StaticBody>, _num_chunks:i64, _chunk_size:i64, _step_height:f32) {
		self.num_chunks = _num_chunks.max(1);
		self.chunk_size = _chunk_size.max(1);
		self.map_size = self.num_chunks as usize * self.chunk_size as usize;
		self.heights = vec![0; self.map_size * self.map_size];
		self.step_height = _step_height;
	}

	#[export]
	fn init_chunks (&mut self, _owner:TRef<StaticBody>) {

		self.chunks.clear();
		self.chunk_vertices.clear();
		self.chunk_indices.clear();

		let chunks = _owner.get_node("Chunks");
		if let Some(chunks) = chunks {
			let chunks = unsafe { chunks.assume_safe() };
			/*for i in 0..chunks.get_child_count() {
				let child = chunks.get_child(i).unwrap();
				let child = unsafe { child.assume_safe() };
				chunks.remove_child(child);
				child.queue_free();
			}*/
			let tree = _owner.get_tree().unwrap();
			let tree = unsafe { tree.assume_safe() };
			//.get_edited_scene_root();


			for i in 0..self.num_chunks as i32 {
				let chunk = MeshInstance::new();
				let x:i32 = i % self.chunk_size as i32;
				let z:i32 = i / self.chunk_size as i32;
				chunk.translate(Vector3::new((x * self.chunk_size as i32) as f32, 0.0, (z * self.chunk_size as i32) as f32));
				chunk.set_owner(tree.get_edited_scene_root());
				chunks.add_child(chunk, true);
				self.chunk_vertices.push(Vec::new());
				self.chunk_indices.push(Vec::new());
			}
		}

		//self.generate_all_meshes(_owner);
		//self.update_editing_collision(_owner);
	}

	#[export]
	fn set_heights (&mut self, _owner:TRef<StaticBody>, heights:TypedArray<i32>) {
		self.heights = (0..heights.len()).map(| i | {
			return heights.get(i);
		}).collect::<Vec<i32>>();
	}

	#[export]
	pub fn clear_terrain (&mut self, _owner:TRef<StaticBody>) {
		for i in 0..self.heights.len() {
			self.heights[i] = 0;
		}
	}

	#[export]
	pub fn get_heights(&self, _owner:TRef<StaticBody>) -> TypedArray<i32>{
		let mut arr:TypedArray<i32> = TypedArray::new();
		for i in 0..self.heights.len() {
			arr.insert(i as i32, self.heights[i]);
		}
		return arr;
	}

	fn set_height_at (&mut self, global_x:i32, global_z:i32, height:i32) {
		let idx:i32 = global_z * self.map_size as i32 + global_x;
		self.heights[idx as usize] = height;
	}

	fn height (&self, global_x:i32, global_z:i32) -> i32 {
		let idx:i32 = global_z * self.map_size as i32 + global_x;
		return self.heights[idx as usize];
	}

	#[export]
	fn get_height (&self, _owner:TRef<StaticBody>, global_x:i32, global_z:i32) -> i32 {
		let x_clamp = (global_x.max(0) as i32).min(self.map_size as i32 - 1);
		let z_clamp = (global_z.max(0) as i32).min(self.map_size as i32 - 1);
		let idx:i32 =  z_clamp * self.map_size as i32 + x_clamp;
		return self.heights[idx as usize];
	}

	#[export]
	pub fn set_step_height (&mut self, _owner:TRef<StaticBody>, value:f32) {
		self.step_height = value;
		self.generate_all_meshes(_owner);
	}

	#[export]
	pub fn get_is_height_locked (&self, _owner:TRef<StaticBody>) -> bool { return self.is_height_locked; }

	#[export]
	pub fn set_is_height_locked (&mut self, _owner:TRef<StaticBody>, value:bool) { self.is_height_locked = value; }

	#[export]
	pub fn get_locked_height (&self, _owner:TRef<StaticBody>) -> i32 { return self.locked_height; }

	#[export]
	pub fn set_locked_height (&mut self, _owner:TRef<StaticBody>, value:i32) { self.locked_height = value; }

	#[export]
	pub fn in_bounds (&mut self, _owner:TRef<StaticBody>, global_x:i32, global_z:i32) -> bool {
		return global_x > -1 && global_z > -1 && global_x < self.map_size as i32 && global_z < self.map_size as i32;
	}

	#[export]
	pub fn get_vertex_count (&self, _owner:TRef<StaticBody>) -> i32 {
		let mut count:usize = 0;
		for vec in self.chunk_vertices.iter() {
			count += vec.len();
		}
		return count as i32;
	}

	fn generate_chunk (&mut self, chunk_id:usize, mesh_instance:TRef<MeshInstance>) {
		generate_chunk_mesh(&self.heights, chunk_id, self.chunk_size as usize, self.num_chunks as usize, &mut self.chunk_vertices[chunk_id], &mut self.chunk_indices[chunk_id]);

        let st = SurfaceTool::new();
        st.begin(4);

		let indices = self.chunk_indices[chunk_id].clone();

		for index in indices.iter() {
			let mut vert = self.chunk_vertices[chunk_id][*index];
			vert.y *= self.step_height;
			st.add_uv(Vector2::new(0.0, 0.0));
			st.add_vertex(vert);
		}

		st.index();
		st.generate_normals(false);		
		st.generate_tangents();

		let mesh_flags = 1 + 2 + 4 + 16 + 256 + 512 + 1024  + 2048 + 8192 + 131072;
        let mesh = st.commit(GodotObject::null(), mesh_flags);
        if let Some(mesh) = mesh {
            mesh_instance.set_mesh(mesh);
        }
	}

    #[export]
    fn generate_chunk_mesh (&mut self, _owner:TRef<StaticBody>, chunk_id:usize) {
		let chunks = _owner.get_node("Chunks");
		if let Some(chunks) = chunks {
			let chunks = unsafe { chunks.assume_safe() };

			let mesh_instance = chunks.get_child(chunk_id as i64);
			if let Some(mesh_instance) = mesh_instance {
				let mesh_instance = unsafe { mesh_instance.assume_safe() };
				let mesh_instance = mesh_instance.cast::<MeshInstance>().unwrap();
				self.generate_chunk(chunk_id, mesh_instance);
			}
		}
    }

	#[export]
	fn generate_all_meshes (&mut self, _owner:TRef<StaticBody>) {
		let chunks = _owner.get_node("Chunks");
		if let Some(chunks) = chunks {
			let chunks = unsafe { chunks.assume_safe() };
			for chunk_id in 0..chunks.get_child_count() {
				let mesh_instance = chunks.get_child(chunk_id as i64).unwrap();
				let mesh_instance = unsafe { mesh_instance.assume_safe() };
				let mesh_instance = mesh_instance.cast::<MeshInstance>().unwrap();
				self.generate_chunk(chunk_id as usize, mesh_instance);
			}
		}
	}


	#[export]
	pub fn end_stroke(&mut self, _owner:TRef<StaticBody>) {
		self.locked_height = -1;
		self.is_drawing = false;
		if self.edited_positions.len() > 0 {
			self.undo_stack.push(self.edited_positions.clone());
		}
		for id in self.chunks_at_seam.iter().cloned().collect::<Vec<usize>>() {
			self.generate_chunk_mesh(_owner, id);
		}
		self.edited_positions.clear();
		self.chunks_at_seam.clear();
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
	pub fn draw_at (&mut self,_owner:TRef<StaticBody>, global_x:i32, global_z:i32, brush_size:Vector2, brush_mode:i32, button_index:i32) -> bool {
		if !self.is_drawing {
			self.is_drawing = true;
			self.edited_positions.clear();
			self.redo_stack.clear();
		}

		let mut dirtied_chunks_set:HashSet<i32> = HashSet::new();
		let brush_height = brush_size.y as i32;

		if brush_mode == 5 && self.in_bounds(_owner, global_x, global_z)  {
			let positions_in_range:Vec<(i32,i32,i32)> = self.get_grid_positions_in_radius(global_x, global_z, brush_size.x as i32 + 1);
			let num_positions_in_range:usize = positions_in_range.len();
			let mut target_height:f32 = 0.0;
			
			if self.is_height_locked {
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
							self.heights[index as usize] = new_height;
							let chunk_id:i32 = ((z / self.chunk_size as i32) * self.num_chunks as i32 + (x / self.chunk_size as i32)) as i32;
							self.check_seam_condition(*x as i32, *z as i32);  
							dirtied_chunks_set.insert(chunk_id);
						}
					}
				}
			}
		}
		else if brush_mode != 5 {
			let positions_in_range:Vec<(i32,i32,i32)> = self.get_grid_positions_in_radius(global_x, global_z, brush_size.x as i32);
			for (x,y,z) in positions_in_range.iter() {
				let chunk_id = self.draw_height_at(*x as i32, *z as i32, brush_mode, brush_height, button_index);
				dirtied_chunks_set.insert(chunk_id);
			}
		}


		let mut do_update_mesh:bool = false;
		for chunk_id in dirtied_chunks_set.iter() {
			if *chunk_id > -1 {
				self.generate_chunk_mesh(_owner, *chunk_id as usize);
				do_update_mesh = true;
			}
		}

		return do_update_mesh;
	}

	fn draw_height_at (&mut self, global_x:i32, global_z:i32, draw_mode:i32, brush_height:i32, button_index:i32) -> i32 {
		if global_x < 0 || global_z < 0 || global_x >= self.map_size as i32 || global_z >= self.map_size as i32 {
			return -1;
		}

		let i:usize = global_z as usize * self.map_size + global_x as usize;
		if self.edited_positions.contains_key(&i) {
			return -1;
		}

		let current_height = self.height(global_x as i32, global_z as i32);
		if self.locked_height == -1 {
			self.locked_height = current_height;
		}
		let mut new_height = current_height;

		if (draw_mode == 0 || draw_mode == 1) &&
			((current_height == self.locked_height && self.is_height_locked) || !self.is_height_locked) {
				if (draw_mode == 0 && button_index == 1) || (draw_mode == 1 && button_index == 2) {
					new_height = current_height + brush_height as i32;
				}
				else if (draw_mode == 1 && button_index == 1) || (draw_mode == 0 && button_index == 2) {
					new_height = (current_height - brush_height as i32).max(0);
				}
		}
		else if draw_mode == 2 {
			if self.is_height_locked && current_height == self.locked_height {
				new_height = current_height + brush_height as i32;
			}
			else if !self.is_height_locked {
				new_height = brush_height as i32;
			}
		}

		else if draw_mode == 3 {
			if self.is_height_locked && current_height < self.locked_height + brush_height as i32 {
				new_height = self.locked_height + brush_height as i32;
			}
			else if !self.is_height_locked && current_height < brush_height as i32 {
				new_height = brush_height as i32;
			}
		}
		else if draw_mode == 4 {
			if self.is_height_locked && current_height > self.locked_height + brush_height as i32 {
				new_height = self.locked_height + brush_height as i32;
			}
			else if !self.is_height_locked && current_height > brush_height as i32 {
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

		let chunk_size:usize = self.chunk_size as usize;
		let num_chunks = self.num_chunks as usize;

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
	fn resize_terrain (&mut self, _owner:TRef<StaticBody>, new_chunk_size:i32, new_num_chunks:i32, x_move:i32, z_move:i32) {
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

		if diff > 0 {
			for z in 0..self.map_size {
				for x in 0..self.map_size {
					let idx = z * self.map_size + x;
					let current_height = self.heights[idx as usize];
					let new_idx = (z as i32 + offset_z) * new_map_size as i32 + x as i32 + offset_x as i32;
					new_heights[new_idx as usize] = current_height;
				}
			}
		}
		else {
			for z in 0..new_map_size {
				for x in 0..new_map_size {
					let idx = (z as i32 - offset_z) * self.map_size as i32 + (x as i32 - offset_x);
					let current_height:i32 = self.heights[idx as i32 as usize];
					let new_idx = z * new_map_size + x;
					new_heights[new_idx as usize] = current_height;
				}
			}
		}

		self.chunk_size = new_chunk_size as i64;
		self.num_chunks = new_num_chunks as i64;
		self.map_size = new_map_size as usize;
		self.heights = new_heights;

		self.edited_positions.clear();
		self.chunks_at_seam.clear();
	}

	#[export]
	fn update_editing_collision (&self, _owner:TRef<StaticBody>) {
		let col = _owner.get_node("EditingShape");
		if let Some(col) = col {
			let col = unsafe { col.assume_safe() };
			let col = col.cast::<CollisionShape>().unwrap();
			let shape = col.shape();
			if let Some(shape) = shape {
				let shape = shape.cast::<HeightMapShape>().unwrap();
				let shape = unsafe { shape.assume_safe() };
				shape.set_map_depth(self.map_size as i64 + 1);
				shape.set_map_depth(self.map_size as i64 + 1);
				shape.set_map_data(self.get_heights_for_collision());
				col.set_shape(shape);
			}
			else {
				let shape = HeightMapShape::new();
				shape.set_map_depth(self.map_size as i64 + 1);
				shape.set_map_depth(self.map_size as i64 + 1);
				shape.set_map_data(self.get_heights_for_collision());
				col.set_shape(shape);
			}
		}
	}

	fn get_heights_for_collision (&self) -> TypedArray<f32> {
		let mut v:Vec<f32> = Vec::new();
		let map_sized_plus = self.map_size + 1;
		for z in 0..map_sized_plus {
			let z_min = z.min(self.map_size - 1);
			for x in 0..map_sized_plus {
				let x_min = x.min(self.map_size - 1);
				let y = self.heights[(z_min * self.map_size + x_min) as usize];
				v.push(y as f32 * self.step_height + 0.5 * self.step_height);
			}
		}
		return TypedArray::from_vec(v);
	}

	#[export]
	pub fn can_undo (&self, _owner:TRef<StaticBody>) -> bool {
		return self.undo_stack.len() > 0;
	}
	#[export]
	pub fn can_redo (&self, _owner:TRef<StaticBody>) -> bool {
		return self.redo_stack.len() > 0;
	}

	#[export]
	pub fn do_undo (&mut self, _owner:TRef<StaticBody>) -> bool {
		let action = self.undo_stack.pop();
		
		if let Some(action) = action {
			let mut new_redo:HashMap<usize,i32> = HashMap::new();
			let mut chunks_to_update:HashSet<usize> = HashSet::new();
			
			self.chunks_at_seam.clear();
			for (index,height) in action.iter() {
				let current_height:i32 = self.heights[*index];
				new_redo.insert(*index, current_height);

				let x = index % self.map_size;
				let z = index / self.map_size;
				let chunk_id:usize = (z / self.chunk_size as usize) * self.num_chunks as usize + (x / self.chunk_size as usize);
				self.heights[*index] = *height;
				self.check_seam_condition(x as i32, z as i32);  
				chunks_to_update.insert(chunk_id);
			}
			for id in chunks_to_update.iter().cloned().collect::<Vec<usize>>() {
				self.generate_chunk_mesh(_owner, id as usize);
			}

			for id in self.chunks_at_seam.iter().cloned().collect::<Vec<usize>>() {
				self.generate_chunk_mesh(_owner, id);
			}
			self.chunks_at_seam.clear();
			self.redo_stack.push(new_redo);
		}
		return self.undo_stack.len() > 0;
	}

	#[export]
	pub fn do_redo (&mut self, _owner:TRef<StaticBody>) -> bool {
		let action = self.redo_stack.pop();
		if let Some(action) = action {
			let mut new_undo:HashMap<usize,i32> = HashMap::new();
			let mut chunks_to_update:HashSet<usize> = HashSet::new();
			
			self.chunks_at_seam.clear();
			for (index,height) in action.iter() {
				let current_height:i32 = self.heights[*index];
				new_undo.insert(*index, current_height);

				let x = index % self.map_size;
				let z = index / self.map_size;
				let chunk_id:usize= (z / self.chunk_size as usize) * self.num_chunks as usize + (x / self.chunk_size as usize);
				self.heights[*index] = *height;
				self.check_seam_condition(x as i32, z as i32);  
				chunks_to_update.insert(chunk_id);
			}
			for id in chunks_to_update.iter().cloned().collect::<Vec<usize>>() {
				self.generate_chunk_mesh(_owner, id);
			}

			for id in self.chunks_at_seam.iter().cloned().collect::<Vec<usize>>() {
				self.generate_chunk_mesh(_owner, id);
			}
			self.chunks_at_seam.clear();
			self.undo_stack.push(new_undo);
		}

		return self.redo_stack.len() > 0;
	}

	#[export]
	fn data_to_file (&self, _owner:TRef<StaticBody>, path:GodotString, settings:Dictionary) {
		let file:Ref<File,Unique> = File::new();
		file.open(path, File::WRITE).unwrap();
		let variables= Dictionary::new();
		variables.insert("heights".to_variant(), self.heights.clone());
		variables.insert("num_chunks".to_variant(), self.num_chunks);
		variables.insert("chunk_size".to_variant(), self.chunk_size);
		variables.insert("step_height".to_variant(), self.step_height);
		variables.insert("settings".to_variant(), settings);
		file.store_var(variables, true);
		file.close();
	}

	#[export]
	fn data_to_obj (&mut self, _owner:TRef<StaticBody>, path:GodotString) {
		self.generate_all_meshes(_owner);

		// Index vertices
		let mut vertex_map:HashMap<Vector3Key,usize> = HashMap::new();
		let mut vertex_list:Vec<Vector3> = Vec::new();
		let mut index_list:Vec<usize> = Vec::new();

		for chunk_id in 0..self.chunks.len() {
			let offset_x = self.chunk_size as i32 * (chunk_id as i32 % self.num_chunks as i32);
			let offset_z = self.chunk_size as i32 * (chunk_id as i32 / self.num_chunks as i32);
			for i in (0..self.chunk_indices[chunk_id].len()).step_by(3) {
				for n in 0..3 {
					let idx = self.chunk_indices[chunk_id][i + n];
					let vert:Vector3 = self.chunk_vertices[chunk_id][idx as usize] + Vector3::new(offset_x as f32, 0.0, offset_z as f32);
					let vert_key:Vector3Key = Vector3Key::new(vert.x, vert.y, vert.z);
					let vert_index = vertex_map.get(&vert_key);
					if let Some(vert_index) = vert_index {
						index_list.push(*vert_index);
					}
					else {
						let new_index = vertex_list.len();
						vertex_map.insert(vert_key, new_index);
						
						vertex_list.push(vert);
						index_list.push(new_index);
					}
				}
			}
		}

		let file:Ref<File,Unique> = File::new();
		file.open(path, File::WRITE).unwrap();
		file.store_line("o terrain");

		for vertex in vertex_list.iter() {
			file.store_line(format!("v {} {} {}", vertex.x, vertex.y * self.step_height, vertex.z));
		}
		file.store_line("s 1");
		for i in (0..index_list.len()).step_by(3) {
			file.store_line(format!("f {} {} {}", index_list[i + 2] + 1, index_list[i + 1] + 1, index_list[i] + 1));
		}
		file.close();
	}
}
