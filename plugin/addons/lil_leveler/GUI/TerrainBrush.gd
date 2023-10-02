tool
class_name TerrainBrush
extends Spatial


const BRUSH_MATERIAL = preload("res://addons/lil_leveler/Materials/brush.material");
const BRUSH_VPAINT_MATERIAL = preload("res://addons/lil_leveler/Materials/brush_vertex_paint.material");
signal brush_changed (property, value)

var size:Vector2 = Vector2.ONE;
var color0:Color = Color.black;
var color1:Color = Color.black;
var opacity:float = 1.0;
var grid_position:Vector3 = Vector3.ZERO;
var position_offset:Vector3 = Vector3.ZERO;

var locked_height:int = -1;
var is_height_locked:bool = false;
var is_drawing:bool = false;

var terrain:TerrainNode = null;

var _cylinder_mesh:CylinderMesh;
var _sphere_mesh:SphereMesh;

enum DrawMode {
	ADD, # Adds height by cursor size
	ERASE, # Adds height by cursor size
	LEVEL, #Sets height equal to the cursor size
	FILL, # Sets height equal to cursor size if height is less than the cursor size
	SHAVE, # Sets height equal to cursor size if height is greater than the cursor size
	SMOOTH, 
}

enum BlendMode {
	MIX,
	ADD,
	MULTIPLY,
	SUBTRACT
}

var draw_mode:int = DrawMode.ADD;
var blend_mode:int = BlendMode.MIX;

func set_mode (mode:int):
	match mode:
		TerrainNode.EditingMode.DRAW_TERRAIN:
			self.visible = true;
			$MeshInstance.mesh = _cylinder_mesh;
			self.update_size();
		TerrainNode.EditingMode.VERTEX_COLORS:
			self.visible = true;
			$MeshInstance.mesh = _sphere_mesh;
			self.update_size();
		TerrainNode.EditingMode.PREVIEW_PUBLISHED:
			self.visible = false;

func _init():
	_cylinder_mesh = CylinderMesh.new();
	_cylinder_mesh.top_radius = 0.5;
	_cylinder_mesh.bottom_radius = 0.5;
	_cylinder_mesh.height = 0.5;
	_cylinder_mesh.rings = 0;
	_cylinder_mesh.radial_segments = 16;
	_cylinder_mesh.material = BRUSH_MATERIAL;
	
	_sphere_mesh = SphereMesh.new();
	_sphere_mesh.radius = 0.5;
	_sphere_mesh.height = 1.0;
	_sphere_mesh.radial_segments = 16;
	_sphere_mesh.rings = 16;
	_sphere_mesh.material = BRUSH_VPAINT_MATERIAL;
	
	var draw_node:MeshInstance = MeshInstance.new();
	self.add_child(draw_node);
	draw_node.set_owner(self);
	self.set_color(self.color0);
	
	
func set_mouse_position (mouse_position:Vector3) -> bool:
	if self.terrain == null:
		return false;
		
	mouse_position -= self.position_offset;
	mouse_position /= self.terrain.scale;
	
	var new_grid_x = round(mouse_position.x);
	var new_grid_z = round(mouse_position.z);
	if self.grid_position.x != new_grid_x || self.grid_position.z != new_grid_z:
		self.grid_position.x = new_grid_x;
		self.grid_position.z = new_grid_z;
		self.update_position();
		return true;
		
	return false;
		

	
func update_size():
	if self.terrain == null:
		return;
	
	if self.get_child(0).mesh is CylinderMesh:
		_cylinder_mesh.top_radius = self.size.x * 0.5 * self.terrain.scale.x;
		_cylinder_mesh.bottom_radius = self.size.x * 0.5 * self.terrain.scale.x;
		_cylinder_mesh.height = self.size.y * self.terrain.scale.y * self.terrain.scale.y;
		self.get_child(0).transform.origin.y = self.size.y  * self.terrain.scale.y / 2.0;
	
	elif self.get_child(0).mesh is SphereMesh:
		_sphere_mesh.radius = self.size.x * 0.5  * self.terrain.scale.x;
		_sphere_mesh.height = self.size.x  * self.terrain.scale.x;
		self.get_child(0).transform.origin.y = 0;
	
	
func update_position():
	if self.terrain == null:
		return;
	
	self.update_size();
		
	var scale_xz:Vector3 = Vector3(self.terrain.scale.x, 1.0, self.terrain.scale.z);
	if self.terrain.editing_mode == TerrainNode.EditingMode.DRAW_TERRAIN:
		
		var grid_y:float = 0.0;
		var height = terrain.height(int(self.grid_position.x), int(self.grid_position.z));
		if height == null:
			return;
		
		if !self.is_height_locked || (self.is_height_locked && self.locked_height == -1):
			if self.draw_mode == DrawMode.ERASE:
				grid_y = height - self.size.y;
			elif self.draw_mode == DrawMode.ADD:
				if self.locked_height == -1:
					grid_y = height;
				else:
					grid_y = height - self.size.y;
			elif self.draw_mode == DrawMode.SMOOTH:
				grid_y = height;
			else:
				if !self.is_height_locked:
					grid_y = 0;
				else:
					grid_y = height;
		
		else:
			if self.draw_mode == DrawMode.ERASE:
				grid_y = self.locked_height - self.size.y;
			else:	
				grid_y = self.locked_height;
		self.grid_position.y = float(grid_y) * self.terrain.scale.y;
		self.transform.origin = self.grid_position * scale_xz + Vector3(0, 0.5 * self.terrain.scale.y, 0) + self.position_offset;
	
	else:
		var height = terrain.height(int(self.grid_position.x), int(self.grid_position.z));
		if height == null:
			return;
		self.transform.origin = self.grid_position * self.terrain.scale + self.position_offset;

func begin_stroke():
	if self.terrain.editing_mode == TerrainNode.EditingMode.DRAW_TERRAIN:
		self.locked_height = terrain.height(int(self.grid_position.x), int(self.grid_position.z));
	else:
		self.locked_height = self.grid_position.y;
	
func end_stroke():
	self.locked_height = -1;
	
func increase_radius():
	self.size.x += 1;
	self.update_size();
	self.update_position();

func decrease_radius():
	self.size.x = max(1, self.size.x - 1);
	self.update_size();
	self.update_position();
	
func increase_height():
	self.size.y += 1;
	self.update_size();
	self.update_position();

func decrease_height():
	self.size.y = max(1, self.size.y - 1);
	self.update_size();
	self.update_position();
	
func swap_colors():
	var tmp = self.color1;
	self.color1 = self.color0;
	self.color0 = tmp;
	self.set_color(self.color0);
	
func set_color (color:Color):
	self.color0 = color;
	var mat:ShaderMaterial = _sphere_mesh.material;
	var color_with_opacity = self.color0;
	color_with_opacity.a *= self.opacity;
	mat.set_shader_param("color", color_with_opacity);
	

func set_opacity (opacity:float):
	self.opacity = opacity;
	var mat:ShaderMaterial = _sphere_mesh.material;
	var color_with_opacity = self.color0;
	color_with_opacity.a *= self.opacity;
	mat.set_shader_param("color", color_with_opacity);

func _on_brush_changed (property:String, value):
	match property:
		"radius":
			self.size.x = value;
			self.update_size();
			self.update_position();
		"height":
			self.size.y = value;
			self.update_size();
			self.update_position();
		"draw_mode":
			self.draw_mode = value;
			self.update_position();
		"color0":
			self.set_color(value);
		"color1":
			self.color1 = value;
		"opacity":
			self.set_opacity(float(value) / 100.0);
		"lock_height":
			self.is_height_locked = value;
		"blend_mode":
			self.blend_mode = value;
