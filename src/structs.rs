use gdnative::{prelude::{Vector3,Vector2,Color}, core_types::TypedArray};

#[derive(Debug,Copy,Clone,PartialEq)]
pub struct Triangle {
    pub points: [Vector3; 3]
}

impl Triangle {
    pub const fn new (p1:Vector3, p2:Vector3, p3:Vector3) -> Self {
        Self {  points: [p1, p2, p3]  }
    }
}

#[derive(Clone,Copy,Hash,Eq,PartialEq,Debug)]
pub struct Vector2Key {
    integral:[u32;2],
    fractional:[u8;2]
}

impl Vector2Key {
    pub fn from_vector2 (v:Vector2) -> Self {
        Vector2Key::new(v.x, v.y)
    }
    pub fn from_vector3 (v:Vector3) -> Self {
        Vector2Key::new(v.x, v.z)
    }
    pub fn new (x:f32, y:f32) -> Self {
        let trunc:[f32;2] = [
            x.trunc() as f32,
            y.trunc() as f32
        ];

        Vector2Key {
            integral: [
                trunc[0] as u32,
                trunc[1] as u32
            ],
            fractional: [
                (x.fract() * 100.0).trunc() as u8,
                (y.fract() * 100.0).trunc() as u8
            ]
        }
    }

    pub fn to_vector2 (&self) -> Vector2 {
        Vector2::new(
            self.integral[0] as f32 + (self.fractional[0] as f32) / 100.0,
            self.integral[1] as f32 + (self.fractional[1] as f32) / 100.0
        )
    }
}



#[derive(Clone,Copy,Hash,Eq,PartialEq,Debug)]
pub struct Vector3Key {
    integral:[u32;3],
    fractional:[u8;3]
}

impl Vector3Key {
    pub fn from_vector3 (v:Vector3) -> Self {
        Vector3Key::new(v.x, v.y, v.z)
    }
    pub fn new (x:f32, y:f32, z:f32) -> Self {
        let trunc:[f32;3] = [
            x.trunc() as f32,
            y.trunc() as f32,
            z.trunc() as f32
        ];

        Vector3Key {
            integral: [
                trunc[0] as u32,
                trunc[1] as u32,
                trunc[2] as u32
            ],
            fractional: [
                (x.fract() * 100.0).trunc() as u8,
                (y.fract() * 100.0).trunc() as u8,
                (z.fract() * 100.0).trunc() as u8
            ]
        }
    }

    pub fn to_vector3 (&self) -> Vector3 {
        Vector3::new(
            self.integral[0] as f32 + (self.fractional[0] as f32) / 100.0,
            self.integral[1] as f32 + (self.fractional[1] as f32) / 100.0,
            self.integral[2] as f32 + (self.fractional[2] as f32) / 100.0 
        )
    }
}

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct Color8 {
	r: u8,
	g: u8,
	b: u8,
	a: u8
}

impl Color8 {
	pub fn new (r:u8, g:u8, b:u8, a:u8) -> Self {
		Color8 { r: r, g: g, b: b, a:a }
	}

	pub fn from_color (color:Color) -> Self {
		Color8::new(
			(color.r * 255.0).clamp(0.0, 255.0) as u8,
			(color.g * 255.0).clamp(0.0, 255.0) as u8,
			(color.b * 255.0).clamp(0.0, 255.0) as u8,
			(color.a * 255.0).clamp(0.0, 255.0) as u8
		)
	}

	pub fn to_color (&self) -> Color {
		Color {
			r: self.r as f32 / 255.0,
			g: self.g as f32 / 255.0,
			b: self.b as f32 / 255.0,
			a: self.a as f32 / 255.0
		}
	}

	pub fn squared_similarity (&self, col:Color8) -> f32 {
		let diff_r = 1.0 - (self.r as i32 - col.r as i32) as f32 / 255.0;
		let diff_g = 1.0 - (self.g as i32 - col.g as i32) as f32 / 255.0;
		let diff_b = 1.0 - (self.b as i32 - col.b as i32) as f32 / 255.0;
		let diff_a = 1.0 - (self.a as i32 - col.a as i32) as f32 / 255.0;

		return diff_r.min(diff_g).min(diff_b).min(diff_a);
	}

	pub fn blend (&self, color:Color8, strength:f32, blend_mode:i32) -> Color8 {
		match blend_mode {
			0 => self.mix(color, strength),
			1 => self.add(color, strength),
			2 => self.multiply(color, strength),
			3 => self.subtract(color, strength),
			_ => self.mix(color, strength)
		}
	}

	pub fn mix (&self, col:Color8, strength:f32) -> Color8 {
		let mix1:f32 = 1.0 - strength;
		let mix2:f32 = strength;
		return Color8::new(
			(self.r as f32 * mix1 + col.r as f32 * mix2).clamp(0.0, 255.0) as u8,
			(self.g as f32 * mix1 + col.g as f32 * mix2).clamp(0.0, 255.0) as u8,
			(self.b as f32 * mix1 + col.b as f32 * mix2).clamp(0.0, 255.0) as u8,
			(self.a as f32 * mix1 + col.a as f32 * mix2).clamp(0.0, 255.0) as u8
		);
	}

	pub fn add (&self, col:Color8, strength:f32) -> Color8 {
		return Color8::new(
			(self.r as f32 + col.r as f32 * strength).clamp(0.0, 255.0) as u8,
			(self.g as f32 + col.g as f32 * strength).clamp(0.0, 255.0) as u8,
			(self.b as f32 + col.b as f32 * strength).clamp(0.0, 255.0) as u8,
			(self.a as f32 + col.a as f32 * strength).clamp(0.0, 255.0) as u8
		);
	}

	pub fn subtract (&self, col:Color8, strength:f32) -> Color8 {
		return Color8::new(
			(self.r as f32 - col.r as f32 * strength).clamp(0.0, 255.0) as u8,
			(self.g as f32 - col.g as f32 * strength).clamp(0.0, 255.0) as u8,
			(self.b as f32 - col.b as f32 * strength).clamp(0.0, 255.0) as u8,
			(self.a as f32 - col.a as f32 * strength).clamp(0.0, 255.0) as u8
		);
	}

	pub fn multiply (&self, col:Color8, strength:f32) -> Color8 {
		let inv_strength = 1.0 - strength;
		let col_r = col.r as f32 / 255.0 + inv_strength * (255.0 - col.r as f32) / 255.0;
		let col_g = col.g as f32 / 255.0 + inv_strength * (255.0 - col.g as f32) / 255.0;
		let col_b = col.b as f32 / 255.0 + inv_strength * (255.0 - col.b as f32) / 255.0;
		let col_a = col.a as f32 / 255.0 + inv_strength * (255.0 - col.a as f32) / 255.0;
		return Color8::new(
			(self.r as f32 * col_r).clamp(0.0, 255.0) as u8,
			(self.g as f32 * col_g).clamp(0.0, 255.0) as u8,
			(self.b as f32 * col_b).clamp(0.0, 255.0) as u8,
			(self.a as f32 * col_a).clamp(0.0, 255.0) as u8
		);
	}
}


pub struct QuadTree {
	size:usize,
    nodes:Vec<Quad>,
}

impl QuadTree {
	pub fn new (_size:usize) -> Self {
        let root = Quad {
            x: 0,
            z: 0,
            size: _size,
            height: 0,
            children: [0; 4],
            parent: 0,
        };

		QuadTree {
			size: _size,
			nodes: vec![root],
		}
	}


    pub fn len(&self) -> usize {
        return self.nodes.len();
    }

    pub fn num_leaf_nodes (&self) -> usize {
        let mut count:usize = 0;
        for node in self.nodes.iter() {
            if !node.is_divided() { count += 1; }
        }
        return count;
    }

    pub fn to_heightmap (&self) -> TypedArray<i32> {
        let mut heights:TypedArray<i32> = TypedArray::new();
        heights.resize(self.size as i32 * self.size as i32);
        for i in 0..(self.size * self.size) {
            let x = i % self.size;
            let z = i / self.size;
            let height = self.value_at(x as i32, z as i32);
            heights.insert(i as i32, height);
        }
        return heights;
    }

    pub fn from_heightmap (&mut self, heights:&TypedArray<i32>) {
        self._heightmap_divide(0, heights);
    }

    fn _heightmap_divide (&mut self, node_index:usize, heights:&TypedArray<i32>) {
        let node = self.nodes[node_index];
        let mut current_height:Option<i32> = None;
        for i in 0..(node.size * node.size) {
            let x = (i % node.size) + node.x;
            let z = (i / node.size) + node.z;
            let height_index = z * self.size + x;
            if current_height.is_none() {
                current_height = Some(heights.get(height_index as i32));
            }
            else if current_height.unwrap() != heights.get(height_index as i32) {
                self._split(node_index);
                for q in 0..4 {
                    self._heightmap_divide(self.nodes[node_index].children[q], heights);
                }
                return;
            }
        }

        if let Some(current_height) = current_height {
            self.nodes[node_index].height = current_height;
        }
    }

    pub fn clear (&mut self) {
        self.nodes[0].children = [0,0,0,0];
        self.nodes = vec![self.nodes[0]];
    }

    pub fn value_at_lod (&self, x:i32, z:i32, lod:usize) -> f32 {
        return self._get_value_lod(0, x, z, lod);
    }

    fn _get_value_lod (&self, index:usize, x:i32, z:i32, lod:usize) -> f32 {
        let node = self.nodes[index];
        if node.contains_xz(x, z) {
            if !node.is_divided() || node.size <= 1 << lod {
                return self._get_child_average(index);
            }
            else {
                let x_local = x - node.x as i32;
                let z_local = z - node.z as i32;
                let half_size = node.size as i32 / 2;
                let quad_index = (z_local / half_size) * 2 + (x_local / half_size);
                let child_index = node.children[quad_index as usize];
                return self._get_value_lod(child_index, x, z, lod);
            }
        }
        return 0.0;
    }
    
    fn _get_child_average (&self, index:usize) -> f32{
        let node = self.nodes[index];
        if node.is_divided() {
            let mut height:f32 = 0.0;
            for q in 0..4 {
                height += 0.25 * self._get_child_average(node.children[q]);
            }
            return height;
        }
        else {
            return node.height as f32;
        }
    }

    pub fn value_at (&self, x:i32, z:i32) -> i32 {
        return self._get_value(0, x, z);
    }

    fn _get_value (&self, index:usize, x:i32, z:i32) -> i32 {
        let node = self.nodes[index];
        if node.contains_xz(x, z) {
            if !node.is_divided() {
                return node.height;
            }
            else {
                let x_local = x - node.x as i32;
                let z_local = z - node.z as i32;
                let half_size = node.size as i32 / 2;
                let quad_index = (z_local / half_size) * 2 + (x_local / half_size);
                let child_index = node.children[quad_index.min(3) as usize];
                return self._get_value(child_index, x, z);
            }
        }
        return 0;
    }

	pub fn insert(&mut self, x:i32, z:i32, height:i32) {
		self._insert( 0, x, z, height);
	}

    fn _remove (&mut self, index:usize) {
        if index == 0  || index >= self.nodes.len() {
            return;
        }

        for i in 0..self.nodes.len() {
            let mut node = self.nodes[i];
            if node.parent >= index {
                self.nodes[i].parent -= 1;
            }
            for q in 0..4 {
                if node.children[q] >= index {
                    self.nodes[i].children[q] -= 1;
                }
            }
        }
        self.nodes.remove(index);
    }

    fn _split (&mut self, node_index:usize) {
        let node = self.nodes[node_index];
        let half_size = node.size / 2;
        for q in 0..4 {

            let q_x = q % 2;
            let q_z = q / 2;

            let new_index = self.nodes.len();
            let quad = Quad {
                x: node.x + (half_size * q_x),
                z: node.z + (half_size * q_z),
                size: half_size,
                height: node.height,
                children: [0; 4],
                parent: node_index,
            };

            self.nodes.push(quad);
            
            ////godot_print!("New child {} ({} {} {}) {}x{}", quad.index, quad.x, quad.height, quad.z, quad.size, quad.size);
            self.nodes[node_index].children[q] = new_index;
        }
    }

    fn _insert (&mut self, node_index:usize, x:i32, z:i32, height:i32) {

        if node_index >= self.nodes.len() {
            return;
        }
        
        let mut node = self.nodes[node_index];

        //godot_print!("");
        //godot_print!("");
        //godot_print!("Inserting ({} {} {}) at node {} {},{} {}x{}", x, height, z, node.index, node.x, node.z, node.size, node.size);
        
        // Doesn't contain the point, so skip
		if !node.contains_xz(x,z) { 

            //godot_print!("Not contained");
            return;
        }

		// Is divided, so add point to the child quad which contains it
		if node.is_divided() {
			for q in 0..4 {
				let child_index = node.children[q];
                if self.nodes[child_index].contains_xz(x,z) {
                    ////godot_print!("Inserted to child");
                    self._insert(child_index, x, z, height);
                    break;
                }
			}
		}

		// Not divided, so check if the point can be inserted
		else {

            ////godot_print!("Not divided");

			// Insert the point anyway if the size is small
            if node.size == 1 || height == node.height {
                //godot_print!("Setting node to this height");
                self.nodes[node_index].height = height;
                self._collapse(self.nodes[node_index].parent);
            }

			// Cannot hold point, so divide into leaf quads
			else if node.size > 1 {
                //godot_print!("Splitting this node");
                let mut contains_this_point:usize = 0;
				
                // Create new leaf quads
				let half_size = node.size / 2;
				for q in 0..4 {

					let q_x = q % 2;
					let q_z = q / 2;

                    let new_index = self.nodes.len();
					let quad = Quad {
						x: node.x + (half_size * q_x),
                        z: node.z + (half_size * q_z),
						size: half_size,
						height: node.height,
						children: [0; 4],
                        parent: node_index,
					};

                    if quad.contains_xz(x, z) {
                        contains_this_point = new_index;
                    }

                    self.nodes.push(quad);
                    
                    //godot_print!("New child {} ({} {} {}) {}x{}", quad.index, quad.x, quad.height, quad.z, quad.size, quad.size);
					self.nodes[node_index].children[q] = new_index;
				}
                
                self._insert(contains_this_point, x, z, height);
			}
		}
    }

    fn _collapse (&mut self, node_index:usize) {
        let node = self.nodes[node_index];
        if node.is_divided() {
            //godot_print!("Trying to collapse {}", node_index);
            let height = self.nodes[node.children[0]].height;
            let mut can_collapse:bool = true;
            for q in 0..4 {
                let child = self.nodes[node.children[q]];
                //godot_print!("Child {}: divided ({}) height ({}).", node.children[q], child.is_divided(), child.height);

                if  child.is_divided() {
                    //godot_print!("Child {} is divided. Cannot collapse", node.children[q]);
                    can_collapse = false;
                    break;
                }

                else if child.height != height {
                    //godot_print!("Child {}'s height does not match. Cannot collapse", node.children[q]);
                    can_collapse = false;
                    break;
                }
            }

            if can_collapse {

                //godot_print!("Can collapse");
                self.nodes[node_index].height = height;
                for q in 0..4 {
                    self._remove( self.nodes[node_index].children[q]);
                }
                self.nodes[node_index].children = [0; 4];
                if node_index != 0 {
                    self._collapse(self.nodes[node_index].parent);
                }
            }
        }
    }

    pub fn get_centers (&self) -> Vec<Vector3> {
        let mut queue:Vec<&Quad> = Vec::new();
		let mut leaf_centers:Vec<Vector3> = Vec::new();
        for i in 0..self.nodes.len() {
            let node = self.nodes[i];
            if !node.is_divided() {
                leaf_centers.push(Vector3::new(
					node.x as f32 + node.size as f32 / 2.0,
					node.height as f32,
					node.z as f32 + node.size as f32 / 2.0,
				));
            }
        }
        return leaf_centers;
    }
}

#[derive(PartialEq,Debug,Copy,Clone)]
struct Quad {
	x:usize,
    z:usize,
	size:usize,
	height:i32,
    parent:usize,
	children:[usize;4],
}


impl Quad {

    fn contains_xz (&self, x:i32, z:i32) -> bool {
        return x >= self.x as i32 && x < self.x as i32 + self.size as i32 &&
            z >= self.z as i32 && z < self.z as i32 + self.size as i32;
    }

	fn is_divided (&self) -> bool {
		return self.children[0] != 0;
	}

    fn to_int_array (&self) -> [i32;2] {
        return [
            self.size as i32,
            self.height
        ];
    }
}



pub struct OctTree {
    color_list:Vec<Color8>,
    nodes: Vec<OctNode>,
    size:usize
}

impl OctTree {
    pub fn new (_size:usize) -> Self {
        let root = OctNode {
            x: 0,
            z: 0,
            y: 0,
            size: _size,
            color: 0,
            children: [0; 8],
            parent: 0,
        };

		OctTree {
			size: _size,
			nodes: vec![root],
            color_list: Vec::new()
		}
	}

    pub fn len(&self) -> usize {
        return self.nodes.len();
    }

    pub fn num_leaf_nodes (&self) -> usize {
        let mut count:usize = 0;
        for node in self.nodes.iter() {
            if !node.is_divided() { count += 1; }
        }
        return count;
    }

    pub fn from_color_list (&mut self, indices:&Vec<usize>, color_list:Vec<Color8>) {
        self.color_list = color_list;
        self._color_list_divide(0, indices);
    }

    fn _color_list_divide (&mut self, node_index:usize, indices:&Vec<usize>) {
        let node = self.nodes[node_index];
        let mut current_index:Option<usize> = None;
        
        // Loop through each position within the node's cube bounds
        for i in 0..(node.size * node.size*node.size) {
            let ny = i / (node.size * node.size);
            let nz = (i - ny * node.size * node.size) / node.size;
            let nx = i % node.size;
           
            // Get the index corresponding to the position within the node
            let y = ny + node.y;
            let z = nz + node.z;
            let x = nx + node.x;
            let idx = y * self.size * self.size + z * self.size + x;
            
            if current_index.is_none() {
                current_index= Some(indices[idx]);
            }
            else if current_index.unwrap() != indices[idx] {
                self._split(node_index);
                for q in 0..4 {
                    self._color_list_divide(self.nodes[node_index].children[q], indices);
                }
                return;
            }
        }

        if let Some(current_index) = current_index {
            self.nodes[node_index].color = current_index;
        }
    }

    pub fn clear (&mut self) {
        self.nodes[0].children = [0;8];
        self.nodes = vec![self.nodes[0]];
    }
    

    pub fn value_at (&self, x:i32, y:i32, z:i32) -> Color8 {
        return self._get_value(0, x, y, z);
    }

    fn _get_value (&self, index:usize, x:i32, y:i32, z:i32) -> Color8 {
        let node = self.nodes[index];
        if node.contains(x, y, z) {
            if !node.is_divided() {
                return self.color_list[node.color];
            }
            else {
                let x_local = x - node.x as i32;
                let z_local = z - node.z as i32;
                let half_size = node.size as i32 / 2;
                let quad_index = (z_local / half_size) * 2 + (x_local / half_size);
                let child_index = node.children[quad_index as usize];
                return self._get_value(child_index, x, y, z);
            }
        }
        return Color8::new(0, 0, 0, 0);
    }

	pub fn insert(&mut self, x:i32, y:i32, z:i32, color8:Color8) {
        let color_list_len = self.color_list.len();
        let mut color = color_list_len;
        for i in 0..color_list_len {
            if self.color_list[i] == color8 {
                color = i;
            }
        }
        if color == color_list_len {
            self.color_list.push(color8);
        }
		self._insert( 0, x, y, z, color);
	}

    fn _remove (&mut self, index:usize) {
        if index == 0  || index >= self.nodes.len() {
            return;
        }

        for i in 0..self.nodes.len() {
            let node = self.nodes[i];
            if node.parent >= index {
                self.nodes[i].parent -= 1;
            }
            for q in 0..8 {
                if node.children[q] >= index {
                    self.nodes[i].children[q] -= 1;
                }
            }
        }
        self.nodes.remove(index);
    }

    fn _split (&mut self, node_index:usize) {
        let node = self.nodes[node_index];
        let half_size = node.size / 2;
        for q in 0..8 {

            let q_y = q / 4;
            let q_z = (q - q_y * 4) / 2;
            let q_x = q % 2;

            let new_index = self.nodes.len();
            let quad = OctNode {
                y: node.y + (half_size * q_y),
                x: node.x + (half_size * q_x),
                z: node.z + (half_size * q_z),
                size: half_size,
                color: node.color,
                children: [0; 8],
                parent: node_index,
            };

            self.nodes.push(quad);
            
            ////godot_print!("New child {} ({} {} {}) {}x{}", quad.index, quad.x, quad.height, quad.z, quad.size, quad.size);
            self.nodes[node_index].children[q] = new_index;
        }
    }

    fn _insert (&mut self, node_index:usize, x:i32, y:i32, z:i32, color:usize) {

        if node_index >= self.nodes.len() {
            return;
        }
        
        let node = self.nodes[node_index];

        //godot_print!("");
        //godot_print!("");
        //godot_print!("Inserting ({} {} {}) at node {} {},{} {}x{}", x, height, z, node.index, node.x, node.z, node.size, node.size);
        
        // Doesn't contain the point, so skip
		if !node.contains(x, y,z) { 

            //godot_print!("Not contained");
            return;
        }

		// Is divided, so add point to the child quad which contains it
		if node.is_divided() {
			for q in 0..8 {
				let child_index = node.children[q];
                if self.nodes[child_index].contains(x, y,z) {
                    ////godot_print!("Inserted to child");
                    self._insert(child_index, x, y, z, color);
                    break;
                }
			}
		}

		// Not divided, so check if the point can be inserted
		else {

            ////godot_print!("Not divided");

			// Insert the point anyway if the size is small
            if node.size == 1 || color == node.color {
                //godot_print!("Setting node to this height");
                self.nodes[node_index].color = color;
                //self._collapse(self.nodes[node_index].parent);
            }

			// Cannot hold point, so divide into leaf quads
			else if node.size > 1 {
                //godot_print!("Splitting this node");
                let mut contains_this_point:usize = 0;


                let half_size = node.size / 2;
                for q in 0..8 {

                    let q_y = q / 4;
                    let q_z = (q - q_y * 4) / 2;
                    let q_x = q % 2;

                    let new_index = self.nodes.len();
                    let child = OctNode {
                        y: node.y + (half_size * q_y),
                        x: node.x + (half_size * q_x),
                        z: node.z + (half_size * q_z),
                        size: half_size,
                        color: node.color,
                        children: [0; 8],
                        parent: node_index,
                    };

                    if child.contains(x, y, z) {
                        contains_this_point = new_index;
                    }

                    self.nodes.push(child);
                    
                    ////godot_print!("New child {} ({} {} {}) {}x{}", quad.index, quad.x, quad.height, quad.z, quad.size, quad.size);
                    self.nodes[node_index].children[q] = new_index;
                }
                
                self._insert(contains_this_point, x, y, z, color);
			}
		}
    }

    fn _collapse (&mut self, node_index:usize) {
        let node = self.nodes[node_index];
        if node.is_divided() {
            //godot_print!("Trying to collapse {}", node_index);
            let color = self.nodes[node.children[0]].color;
            let mut can_collapse:bool = true;
            for q in 0..4 {
                let child = self.nodes[node.children[q]];
                //godot_print!("Child {}: divided ({}) height ({}).", node.children[q], child.is_divided(), child.height);

                if  child.is_divided() {
                    //godot_print!("Child {} is divided. Cannot collapse", node.children[q]);
                    can_collapse = false;
                    break;
                }

                else if child.color != color {
                    //godot_print!("Child {}'s height does not match. Cannot collapse", node.children[q]);
                    can_collapse = false;
                    break;
                }
            }

            if can_collapse {

                //godot_print!("Can collapse");
                self.nodes[node_index].color = color;
                for q in 0..8 {
                    self._remove( self.nodes[node_index].children[q]);
                }
                self.nodes[node_index].children = [0; 8];
                if node_index != 0 {
                    self._collapse(self.nodes[node_index].parent);
                }
            }
        }
    }
}

#[derive(PartialEq,Debug,Copy,Clone)]
pub struct OctNode {
    x:usize,
    y:usize,
    z:usize,
    size:usize,
    parent:usize,
    children:[usize;8],
    color:usize, // Index of color in list
}


impl OctNode {
    pub fn is_divided(&self) -> bool {
        return self.children[0] != 0;
    }

    pub fn contains (&self, _x: i32, _y:i32, _z:i32) -> bool {
        return  _x >= self.x as i32 && _x < (self.x + self.size) as i32 &&
                _y >= self.y as i32 && _y < (self.y + self.size) as i32 &&
                _z >= self.z as i32 && _z < (self.z + self.size) as i32
    }
}

