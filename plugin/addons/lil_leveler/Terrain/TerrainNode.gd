tool
class_name TerrainNode
extends StaticBody

# Plugin-only signal
signal terrain_property_changed (property, value)


const DEBUG:bool = true;

# Plugin-only enum
enum EditingMode {
	DRAW_TERRAIN,
	VERTEX_COLORS,
	PREVIEW_PUBLISHED,
}

# You can add presets by adding a name to the TerrainMaterialType enum.
# HOWEVER -- Leave CUSTOM as the last enum.
# Then add a preload statement with the path to the new material to the end of the MATERIALS array

enum TerrainMaterialType {
	EDITOR_LIGHTGRAY,
	EDITOR_ANTIQUE,
	EDITOR_MARINE,
	PAINT_VERTEX_COLORS,
	CUSTOM
}

const MATERIALS = [
	preload("../Materials/grid_mesh_lightgray.material"),
	preload("../Materials/grid_mesh_antique.material"),
	preload("../Materials/grid_mesh_marine.material"),
	preload("../Materials/grid_mesh_vertex_color.material")
]

# Preloading native script for terrain generation
const TerrainUtils = preload("TerrainUtils.gdns");

# Tells the editor plugin whether terrain is being added or vertex colors are being modified
export (EditingMode) var editing_mode:int = 0 setget set_editing_mode;

# For editing purposes, the terrain is split up into multiple sections known as chunks.
# Each chunk has the size N x N, where N is chunk size.
# The map can be composed of M x M chunks, where M is the number of chunks.
# You can set chunk_size and num_chunks using the inspector or using the "Resize" button under "More" in the toolbar.
# By manually setting values in the inspector. Terrain will be expanded/contracted equally on all sides.
# Use "Resize" for better control over resizing the terrain.
export var chunk_size:int = 16 setget set_chunk_size;
export var num_chunks:int = 4 setget set_num_chunks;


# Set material preset or use custom material for terrain shading
export (TerrainMaterialType) var material_type = 0 setget set_material_type;

# Set a custom shader or spatial material for the terrain
export var custom_material:Material = null setget set_custom_material;

# Check true for smooth shading or set false for flat shading.
# Flat shading may have performance cost because vertices are duplicated
export var shade_smooth:bool = true setget set_shade_smooth;

export var saved_heights:PoolIntArray;
export var saved_color_list:PoolColorArray;
export var saved_vertex_colors:PoolIntArray;
export var saved_mesh:ArrayMesh;

# The wrapper object and interface for the backend of terrain generation.
var terrain_utils:Resource;

# The material for the terrain and terrain chunks
var material:Material = null;

# The total size of the terrain
var map_size:int = 64;

# Set when the node begins _ready() to prevent certain parts of the setter functions to activate
var ready:bool = false;

# Holds vertex counts. Currently has no uses.
var vertex_count:int = 0;

func get_class() -> String:
	return "TerrainNode"

func _ready():
	self.set_process(!Engine.editor_hint);
	
	if !Engine.editor_hint:
		self.editing_mode = EditingMode.PREVIEW_PUBLISHED;
	
	self.ready = true;
	if self.terrain_utils == null:
		self.terrain_utils = Resource.new();
		self.terrain_utils.resource_local_to_scene = true;
	
	self.terrain_utils.set_script(TerrainUtils);
	
	if !self.has_node("EditingShape"):
		var editing_shape = CollisionShape.new();
		editing_shape.name = "EditingShape"
		editing_shape.shape = HeightMapShape.new();
		editing_shape.visible = false;
		self.add_child(editing_shape);
		editing_shape.set_owner(self);
		
	if !self.has_node("Chunks"):
		var chunks = Spatial.new();
		chunks.name = "Chunks";
		self.add_child(chunks);
		chunks.set_owner(self);
			
	if !self.has_node("PublishedShape"):
		var published_shape = CollisionShape.new();
		published_shape.name = "PublishedShape";
		published_shape.visible = false;
		published_shape.shape = null;
		self.add_child(published_shape);
		published_shape.set_owner(self);
	
	if !self.has_node("PublishedMesh"):
		var published_mesh = MeshInstance.new();
		published_mesh.name = "PublishedMesh";
		self.add_child(published_mesh);
		published_mesh.set_owner(self);
		published_mesh.mesh = ArrayMesh.new();
		
	if self.material_type < MATERIALS.size():
		self.material = MATERIALS[self.material_type];
	else:
		self.material = self.custom_material;
		
	self.update_materials();
	self.init();
	self.update_editing_mode();
	
# Initializes the terrain using terrain_utils
func init():
	self.map_size = self.chunk_size * self.num_chunks;
	
	if self.terrain_utils && self.terrain_utils.has_method("init_params") && Engine.editor_hint:
		self.terrain_utils.call("set_shade_smooth", self.shade_smooth);
		self.terrain_utils.call("init_params", self.num_chunks, self.chunk_size, self.saved_heights, self.saved_color_list, self.saved_vertex_colors);
		self.terrain_utils.call("update_terrain"); # Contains function for updating all chunks and updating terrain vertex data (without creating the terrain mesh)
		self.generate_published_mesh(); # Creates terrain mesh and its corresponding collision shape
	
	elif !Engine.editor_hint:
		$PublishedMesh.mesh = self.saved_mesh;
		$PublishedShape.shape = $PublishedMesh.mesh.create_trimesh_shape();


func save_to_file():
	if self.terrain_utils != null:
		self.saved_heights = self.terrain_utils.call("get_heights");
		self.saved_color_list = self.terrain_utils.call("get_color_list");
		self.saved_vertex_colors= self.terrain_utils.call("get_vertex_colors");
		self.terrain_utils.call("set_array_mesh", $PublishedMesh);
		self.saved_mesh = $PublishedMesh.mesh;
		print("Saving terrain");
		self.property_list_changed_notify();
	else:
		print("Error saving.")
	#print("Saving data to file: ", self.data_path);
	#self.terrain_utils.call("data_to_file", self.data_path, {});

func update_editing_mode ():
	match self.editing_mode:
		EditingMode.DRAW_TERRAIN:
			if $Chunks.get_child_count() == 0:
				self.init_chunks();
			
			self.update_editing_collision_shape();
			$EditingShape.disabled = false;
			$PublishedShape.disabled = true;
			$PublishedMesh.visible = false;
			$Chunks.visible = true;
			
		EditingMode.VERTEX_COLORS:
			if $Chunks.get_child_count() == 0:
				self.init_chunks();
			$EditingShape.disabled = true;
			$PublishedShape.disabled = false;
			$PublishedMesh.visible = false;
			$Chunks.visible = true;
		
		EditingMode.PREVIEW_PUBLISHED:
			$EditingShape.disabled = true;
			$PublishedShape.disabled = false;
			$PublishedMesh.visible = true;
			$Chunks.visible = false;
	
	self.update_materials();
	self.property_list_changed_notify();

# Helper function to set materials on all chunks or pubished mesh
func update_materials():
	if self.has_node("Chunks"):
		for child in $Chunks.get_children():
			child.material_override = self.material;
	if self.has_node("PublishedMesh"):
		$PublishedMesh.material_override = self.material;
		
# Initializes the chunks in a separate function because we might not need to create chunks
#	such as if the mode is published/preview
func init_chunks ():
	if self.has_node("Chunks") && self.terrain_utils && self.terrain_utils.has_method("init_chunk_meshes"):
		for child in $Chunks.get_children():
			$Chunks.remove_child(child);
			child.queue_free();
		yield(get_tree(), "idle_frame");
		yield(get_tree(), "idle_frame");
		var chunk_index = 0;
		for z in range(self.num_chunks):
			for x in range(self.num_chunks):
				var chunk:MeshInstance = MeshInstance.new();
				chunk.name = "Chunk_" + str(x) + "_" + str(z) 
				chunk.translate(Vector3(x * self.chunk_size, 0, z * self.chunk_size));
				$Chunks.add_child(chunk);
				chunk.set_owner(self);
				chunk_index += 1;
		self.terrain_utils.call("init_chunk_meshes", $Chunks.get_children());
		self.update_materials();
		
# Updates the editing collision shape, which is a heightmap shape	
func update_editing_collision_shape():
	if self.editing_mode == EditingMode.DRAW_TERRAIN:
		if self.has_node("EditingShape"):
			var shape:HeightMapShape = $EditingShape.shape;
			shape.map_depth = self.map_size + 1;
			shape.map_width = self.map_size + 1;	
			shape.map_data = self.terrain_utils.call("get_heights_for_collision");
			$EditingShape.transform.origin = Vector3(1, 0, 1) * self.map_size / 2.0;


func generate_published_mesh():
	if self.terrain_utils != null && self.is_inside_tree():
		self.terrain_utils.call("set_array_mesh", $PublishedMesh);
		$PublishedShape.shape = $PublishedMesh.mesh.create_trimesh_shape();
		self.saved_mesh = $PublishedMesh.mesh;
		
func update_published_collision_shape():
	if self.is_inside_tree():
		if self.saved_mesh != null:
			$PublishedShape.shape = $PublishedMesh.mesh.create_trimesh_shape();
		else:
			self.generate_published_mesh();
		
# Wrapper function for drawing/erasing terrain. Does the following:
#	- Updates the heights according to the brush's mode
#	- Updates the affected chunks and regenerates the chunk mesh
func draw_at (grid_x:int, grid_z:int, brush_size:Vector2, brush_mode:int, button_index:int=BUTTON_LEFT, is_height_locked:bool=false, locked_height:int=-1):
	self.terrain_utils.call("draw_at", grid_x, grid_z, brush_size, brush_mode, button_index, is_height_locked, locked_height);

# Wrapper function for vertex painting.
# Essentially changes the vertex colors within the radius of the brush position of each chunk mesh without regenerating each chunk mesh.
# See TerrainBrush.gd for blend modes.
func paint_vertex (position:Vector3, brush_radius:float, color:Color, opacity:float, blend_mode:int):
	self.terrain_utils.call("paint_vertex", position, brush_radius, color, opacity, blend_mode);
	
# Wrapper function for when drawing/painting is stopped.
# terrain_utils.end_stroke() Does the following for terrain drawing:
#	- Updates any adjacent chunks for fixing seams
#	- Updates the entire terrain's vertex data
func end_stroke ():
	if self.editing_mode == EditingMode.DRAW_TERRAIN:
		self.terrain_utils.call("end_stroke");
		self.update_editing_collision_shape();
	elif self.editing_mode == EditingMode.VERTEX_COLORS:
		self.terrain_utils.call("end_paint_stroke");
	self.property_list_changed_notify();

# Sets vertex colors by their slope (ie. angle in radians from UP vector)
# See TerrainBrush.gd for blend modes.
#### Deprecated for now
#func color_by_slope (min_slope:float, max_slope:float, color:Color, strength:float, blend_mode:int):
#	self.terrain_utils.call("color_by_slope", min_slope, max_slope, color, strength, blend_mode);

# Replaces or modifies a select color within a similarity threshold (0.0 is least stringent; 1.0 is exact match);
# See TerrainBrush.gd for blend modes.	
#### Deprecated for now
#func replace_color (old_color:Color, similarity:float, new_color:Color, strength:float, blend_mode:int):
	#self.terrain_utils.call("replace_color", old_color, similarity, new_color, strength, blend_mode);
	
func do_undo():
	if self.terrain_utils && self.terrain_utils.has_method("do_undo"):
		self.terrain_utils.call("do_undo");
		self.update_editing_collision_shape();

func do_redo():
	if self.terrain_utils && self.terrain_utils.has_method("do_redo"):
		self.terrain_utils.call("do_redo");
		self.update_editing_collision_shape();
	
# Expands/contracts the terrain. X/Z coord correspond to 3x3 cells, where (1,1) is the center
# When coord is (1,1), terrain is expanded/contracted equally across all edges
# When coord is (0,0) (this is the top left), terrain is expanded/contracted to the right and down
# When coord is (1,0) (this is the top center), the terrain is expanded/contracted down and between the left and right equally
func resize_canvas (new_chunk_size:int, new_num_chunks:int, x_coord:int=1, z_coord:int=1):
	if self.terrain_utils && self.terrain_utils.has_method("resize_terrain"):
		self.terrain_utils.call("resize_terrain", new_chunk_size, new_num_chunks, x_coord, z_coord);
		self.num_chunks = new_num_chunks;
		self.chunk_size = new_chunk_size;
		self.map_size = new_num_chunks * new_chunk_size;
		self.update_editing_collision_shape();
		self.init_chunks();
		self.property_list_changed_notify();

# Resets all terrain heights and vertex colors;	
func clear():
	self.saved_heights = PoolIntArray();
	self.saved_vertex_colors = PoolIntArray();
	self.saved_color_list = PoolColorArray();
	self.terrain_utils.call("init_params", self.num_chunks, self.chunk_size, self.saved_heights, self.saved_color_list, self.saved_vertex_colors);
	self.terrain_utils.call("update_terrain");
	$PublishedMesh.mesh = ArrayMesh.new();
	self.update_editing_collision_shape();
	self.init_chunks();

# Exports the terrain mesh to a .OBJ file 
# OBJ doesn't support vertex colors, however
func export_mesh(path:String):
	self.terrain_utils.call("data_to_obj", path);

# Helper function to get height at x,z position.
# (x,z) must be local to the terrain
func height (x:int, z:int) -> int:
	if self.terrain_utils != null && self.terrain_utils.has_method("get_height"):
		return self.terrain_utils.call("get_height", x, z);
	return 0;
	
func color (x:int, z:int) -> Color:
	return self.terrain_utils.call("get_color", x, z);

"""
	Setter/Getter functions below
"""

func set_editing_mode (val:int):
	if val != editing_mode:
		var do_generate_published_mesh:bool = editing_mode == EditingMode.DRAW_TERRAIN || val == EditingMode.PREVIEW_PUBLISHED;
		editing_mode = val;
		if ready:
					
			if do_generate_published_mesh:
				terrain_utils.call("update_terrain");
				generate_published_mesh();
			update_editing_mode();
			
			if material_type != TerrainMaterialType.CUSTOM:
				if editing_mode == EditingMode.DRAW_TERRAIN:
					set_material_type(TerrainMaterialType.EDITOR_LIGHTGRAY);
				elif editing_mode == EditingMode.VERTEX_COLORS:
					set_material_type(TerrainMaterialType.PAINT_VERTEX_COLORS);
					
		emit_signal("terrain_property_changed", "editing_mode", val);
	
func set_chunk_size (val:int):
	val = max(4, val);
	chunk_size = val;
	if val != chunk_size && editing_mode == EditingMode.DRAW_TERRAIN && ready:
		resize_canvas(chunk_size, num_chunks, 1, 1);
	
func set_num_chunks (val:int):
	val = max(1, val);
	num_chunks = val;
	if val != num_chunks && editing_mode == EditingMode.DRAW_TERRAIN && ready:
		resize_canvas(chunk_size, num_chunks, 1, 1);

func set_material_type (val:int):
	if val != material_type:
		if ready:
			if val < MATERIALS.size():
				material_type = val;
				material = MATERIALS[material_type];
				update_materials();
			elif val  == TerrainMaterialType.CUSTOM && custom_material:
				material_type = val;
				material = custom_material;
				update_materials();
		material_type = val;
	
func set_custom_material (val:Material):
	if custom_material != val:
		custom_material = val;
		if material != null:
			set_material_type(TerrainMaterialType.CUSTOM);
			update_materials();
	
func set_shade_smooth (val:bool):
	if val != shade_smooth:
		shade_smooth = val;
		if terrain_utils && ready:
			terrain_utils.call("set_shade_smooth", val);
			if editing_mode == EditingMode.PREVIEW_PUBLISHED:
				terrain_utils.call("set_array_mesh", $PublishedMesh);
			elif editing_mode != EditingMode.PREVIEW_PUBLISHED:
				terrain_utils.call("generate_all_meshes");
	
