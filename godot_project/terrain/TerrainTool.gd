class_name TerrainTool
extends MeshInstance




export var terrain_data:Resource;
export var step_height:float = 0.5;

export var env_theme:Resource;

var paused:bool = false;

var cursor_active:bool = true;
var cursor_size:Vector2 = Vector2.ONE;
var height_locked:bool = false;
var prev_height:int = -1;

export var num_chunks:int = 2;
export var chunk_size:int = 8;

var cursor_grid_position:Vector3;

var current_file_path:String = "";

enum DrawMode {
	ADD, # Adds height by cursor size
	ERASE, # Adds height by cursor size
	LEVEL, #Sets height equal to the cursor size
	FILL, # Sets height equal to cursor size if height is less than the cursor size
	SHAVE, # Sets height equal to cursor size if height is greater than the cursor size
	SMOOTH, 
}

onready var themes = [
	preload("res://themes/default.tres"),
	preload("res://themes/antique.tres"),
	preload("res://themes/digital.tres"),
	preload("res://themes/marine.tres"),
	preload("res://themes/night.tres"),
	preload("res://themes/sunset.tres"),
]

var draw_mode:int = DrawMode.ADD;
export var map_size:int = 8;

export var init_terrain:bool = false setget _do_init_terrain;
func _do_init_terrain(val:bool):
	if val:
		init();

func _ready():
	Events.connect_signals(self,["mouse_event", "property_change", "exec_function", "query_map_size"]);
	self.init();
	self.load_theme();
	for theme in self.themes:
		var theme_name:String = (theme.resource_path as String).get_file().split(".")[0].replace("_", " ").capitalize();
		$CanvasLayer/HUD.add_theme(theme_name);
		

var update_cooldown:float = 0.0;
func _process(delta):
	if update_cooldown <= 0.0:
		self.update_cursor();
		update_cooldown = 0.05;
	update_cooldown -= delta;

func update_cursor_size():
	if $Cursor.get_child(0).mesh is CubeMesh:
		var cubemesh:CubeMesh = $Cursor.get_child(0).mesh;
		cubemesh.size.x = self.cursor_size.x;
		cubemesh.size.z = self.cursor_size.x;
		cubemesh.size.y = self.cursor_size.y * self.step_height;
	if $Cursor.get_child(0).mesh is CylinderMesh:
		var cylindermesh:CylinderMesh = $Cursor.get_child(0).mesh;
		cylindermesh.top_radius = self.cursor_size.x * 0.5;
		cylindermesh.bottom_radius = self.cursor_size.x * 0.5;
		cylindermesh.height = self.cursor_size.y * self.step_height;
	$Cursor.get_child(0).transform.origin.y = self.cursor_size.y * self.step_height / 2.0;
		
func update_cursor():
	var grid_y:float = 0.0;
	var height:int = self.height(int(self.cursor_grid_position.x), int(self.cursor_grid_position.z));
	var is_height_locked:bool = self.is_height_locked();
	var locked_height:int = self.locked_height();
	
	if !is_height_locked || (is_height_locked && locked_height == -1):
		if self.draw_mode == DrawMode.ERASE:
			grid_y = height - self.cursor_size.y;
		elif self.draw_mode == DrawMode.ADD:
			if locked_height == -1:
				grid_y = height;
			else:
				grid_y = height - self.cursor_size.y;
		elif self.draw_mode == DrawMode.SMOOTH:
			grid_y = height;
		else:
			if !is_height_locked:
				grid_y = 0;
			else:
				grid_y = height;
	else:
		if self.draw_mode == DrawMode.ERASE:
			grid_y = locked_height - self.cursor_size.y;
		else:	
			grid_y = locked_height;
	
	self.cursor_grid_position.y = float(grid_y) * self.step_height;
	$Cursor.global_transform.origin = self.cursor_grid_position + Vector3(-0.5, self.step_height / 2.0, -0.5);

func locked_height () -> int:
	return self.terrain_data.call("get_locked_height");
func is_height_locked () -> bool:
	return self.terrain_data.call("get_is_height_locked");
func height (x:int, z:int) -> int:
	return self.terrain_data.call("get_height", x, z);

func update_vertex_count():
	var total_v = self.terrain_data.call("get_vertex_count");
	$CanvasLayer/HUD/VertexCount.text = "Vertex count: " + str(total_v);

func init():
	self.init_params();
	self.init_chunks();

func init_params ():
	self.map_size = self.chunk_size * self.num_chunks;
	self.terrain_data.call("init_params", self.num_chunks, self.chunk_size, self.step_height);
	self.update_collision_shape();

func init_chunks ():
	$CanvasLayer/HUD.set_map_params(self.num_chunks, self.chunk_size);
	for child in $Chunks.get_children():
		child.queue_free();
	get_tree().paused = true;
	yield(get_tree(), "idle_frame");
	yield(get_tree(), "idle_frame");
	get_tree().paused = false;
	var chunk_index = 0;
	for z in range(self.num_chunks):
		for x in range(self.num_chunks):
			var chunk:MeshInstance = MeshInstance.new();
			chunk.material_override = self.material_override;
			chunk.translate(Vector3(x * self.chunk_size, 0, z * self.chunk_size));
			$Chunks.add_child(chunk);
			chunk_index += 1;
	self.terrain_data.call("init_chunks", $Chunks.get_children());
	self.update_vertex_count();

func update_all_chunks():
	self.terrain_data.call("generate_all_meshes");
	self.update_vertex_count();

func update_collision_shape():
	$StaticBody.transform.origin.x = (self.chunk_size * self.num_chunks + 1) / 2;
	$StaticBody.transform.origin.z = (self.chunk_size * self.num_chunks + 1) / 2;
	var shape:HeightMapShape = $StaticBody/CollisionShape.shape;
	shape.map_depth = self.chunk_size * self.num_chunks + 1;
	shape.map_width = self.chunk_size * self.num_chunks + 1;	
	shape.map_data = self.terrain_data.call("get_heights_for_collision");

func save_file (path:String):
	self.terrain_data.call("data_to_file", path, $CanvasLayer/HUD.settings);
	self.current_file_path = path;
	$CanvasLayer/HUD.current_file_path = path;
	
func open_file (path:String):
	var file:File = File.new();
	if file.file_exists(path):
		file.open(path, File.READ);
		var variables:Dictionary = file.get_var(true);
		file.close(); 
		
		if variables.has_all(["chunk_size", "num_chunks", "step_height", "heights"]):
			self.chunk_size = int(variables['chunk_size']);
			self.num_chunks = int(variables['num_chunks']);
			self.step_height = float(variables['step_height']);
			self.init_params();
			self.terrain_data.call("set_heights", PoolIntArray(variables['heights']));
			self.init_chunks();
			self.update_collision_shape();
			
			self.current_file_path = path;
			$CanvasLayer/HUD.current_file_path = path;
			
			if variables.has("settings"):
				$CanvasLayer/HUD.load_settings(variables['settings']);
			
		else:
			OS.alert("Can't read file.");
		
func export_mesh (path:String):
	self.terrain_data.call("data_to_obj", path);

func _on_mouse_event (event_type, grid_position, button_index:int=-1):
	if !self.cursor_active || !self.terrain_data.call("in_bounds", int(grid_position.x), int(grid_position.z)):
		return;
		
	match event_type:
		"move":
			self.cursor_grid_position = grid_position;
		"draw":
			self.terrain_data.call("draw_at", int(grid_position.x), int(grid_position.z), self.cursor_size, self.draw_mode, button_index);
		"release":
			self.terrain_data.call("end_stroke");
			self.update_vertex_count();
			update_collision_shape();
			$CanvasLayer/HUD.update_undo_redo(self.terrain_data.call("can_undo"),self.terrain_data.call("can_redo"));

func _on_property_change (propname:String, value):
	if self.get(propname) != null:
		self.set(propname, value);
		
	match propname:
		"step_height":
			self.terrain_data.call("set_step_height", value);
			self.update_collision_shape();
		"is_height_locked":
			self.terrain_data.call("set_is_height_locked", value);
		"cursor_width":
			self.cursor_size.x = max(1, value);
			self.update_cursor_size();
		"cursor_height":	
			self.cursor_size.y = max(1, value);
			self.update_cursor_size();
		"paused":
			self.set_process(!value);



func _on_exec_function (func_name, value):
	match func_name:
		"clear_all":
			self.init();

		"open_file":
			self.open_file(value);
		
		"save_file":
			self.save_file(value);
			
		"export_mesh":
			self.export_mesh(value);
			
		"undo_redo":
			if value == "undo":
				self.terrain_data.call("do_undo");
				self.update_collision_shape();
			elif value == "redo":
				self.terrain_data.call("do_redo");
				self.update_collision_shape();
			$CanvasLayer/HUD.update_undo_redo(self.terrain_data.call("can_undo"),self.terrain_data.call("can_redo"));
			
		"resize_terrain":
			var new_chunk_size:int = (value as Rect2).position.x;
			var new_num_chunks:int = (value as Rect2).position.y;
			var x_amount:int = (value as Rect2).size.x;
			var z_amount:int = (value as Rect2).size.y;
			self.terrain_data.call("resize_terrain", new_chunk_size, new_num_chunks, x_amount, z_amount);
			self.num_chunks = new_num_chunks;
			self.chunk_size = new_chunk_size;
			self.map_size = new_num_chunks * new_chunk_size;
			self.update_collision_shape();
			self.init_chunks();
		
		"load_theme":
			if value < self.themes.size():
				var theme = self.themes[value];
				if theme != self.env_theme:
					self.env_theme = theme;
					self.load_theme();

func _on_query_map_size ():
	Events.emit_signal("return_map_size", self.num_chunks, self.chunk_size);

func load_theme():
	if self.env_theme != null:
		self.env_theme.generate_environment();
		$WorldEnvironment.environment = self.env_theme.environment;
		var mat:ShaderMaterial = self.material_override;
		mat.set_shader_param("show_grid", self.env_theme.mesh_show_grid);
		mat.set_shader_param("main_color", self.env_theme.mesh_main_color);
		mat.set_shader_param("grid_color", self.env_theme.mesh_grid_color);
		mat.set_shader_param("grid_width", self.env_theme.mesh_grid_width / 10.0);
		$DirectionalLight.light_color = self.env_theme.light_color;
		var mat2:ShaderMaterial = $Cursor/MeshInstance.get_surface_material(0);
		mat2.set_shader_param("top_color", self.env_theme.cursor_top_color);
		mat2.set_shader_param("bottom_color", self.env_theme.cursor_bottom_color);
