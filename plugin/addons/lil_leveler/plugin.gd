tool
extends EditorPlugin

const BRUSH = preload("GUI/TerrainBrush.gd");
const TERRAIN_NODE = preload("Terrain/TerrainNode.gd");
const TOOLBAR = preload("GUI/TerrainToolBar.tscn");
const MENU = preload("GUI/TerrainMenu.tscn");

var _terrain_brush:TerrainBrush = null;
var _terrain:TerrainNode = null;
var _camera:Camera = null;
var _terrain_toolbar:TerrainToolbar = null;
var _menu = null;

var _import_tdata;

var draw_button_index:int = -1;

const input_directions = {
	"forward": Vector3.FORWARD,
	"back": Vector3.BACK,
	"right": Vector3.RIGHT,
	"left": Vector3.LEFT,
	"up": Vector3.UP,
	"down": Vector3.DOWN,
}

func _enter_tree():
	add_custom_type("TerrainNode", "StaticBody", TERRAIN_NODE, preload("icon.png"))
	
	_terrain_brush = BRUSH.new();
	_terrain_brush.visible = false;
	self.add_child(_terrain_brush);
	
	_menu = MENU.instance();
	add_control_to_container(EditorPlugin.CONTAINER_SPATIAL_EDITOR_MENU, _menu);
	
	_terrain_toolbar = TOOLBAR.instance();
	add_control_to_container(EditorPlugin.CONTAINER_SPATIAL_EDITOR_MENU, _terrain_toolbar);
	_terrain_toolbar.visible = false;
	_terrain_toolbar.connect("brush_changed", _terrain_brush, "_on_brush_changed");
	_terrain_toolbar.connect("show_popup", _menu, "_on_show_popup");
	_menu.connect("menu_action", self, "_on_menu_action");
	_menu.connect("menu_action", _terrain_toolbar, "_on_menu_action");
	
func _exit_tree():
	remove_custom_type("TerrainNode")
	
	self.edit(null);
	if _terrain_toolbar != null:
		_terrain_toolbar.free();
	
	if _terrain_brush != null:
		self.remove_child(_terrain_brush);
		_terrain_brush.free();

		
func make_visible(visible):
	if !visible:
		self.edit(null);
	return visible;
	
func clear():
	self.edit(null);
	
func apply_changes():
	if _terrain:
		_terrain.save_to_file();	
	else:
		if self.get_editor_interface().get_edited_scene_root() != null:
			var queue = [self.get_editor_interface().get_edited_scene_root()];
			while queue.size() > 0:
				var current = queue.pop_back();
				if current.get_class() == "TerrainNode":
					current.save_to_file();
				else:
					for child in current.get_children():
						queue.append(child);
	
func handles(object):
	var handling = false;
	if object != null && object is Node:
		if object.get_class() == "TerrainNode":
			handling = true;
		else:
			_terrain_toolbar.visible = false;
			_terrain_brush.visible = false;
		
	return handling;

func edit(object):
	if object != null && object.get_class() == "TerrainNode" && object != _terrain:
		
		if _terrain != null && _terrain.is_connected("terrain_property_changed", self, "_on_terrain_property_changed"):
			_terrain.disconnect("terrain_property_changed", self, "_on_terrain_property_changed");
		
		if !object.is_connected("terrain_property_changed", self, "_on_terrain_property_changed"):
			object.connect("terrain_property_changed", self, "_on_terrain_property_changed");
			
		_terrain = object;
		_terrain_brush.terrain = object;
		_terrain_brush.position_offset = object.global_transform.origin;
		_terrain_toolbar.set_mode(_terrain.editing_mode);
		_terrain_brush.set_mode(_terrain.editing_mode);	
		_menu.load_map_params(_terrain.chunk_size, _terrain.num_chunks);

	else:
		
		if _terrain != null && _terrain.is_connected("terrain_property_changed", self, "_on_terrain_property_changed"):
			_terrain.disconnect("terrain_property_changed", self, "_on_terrain_property_changed");
		
		_terrain_brush.terrain = object;
		_terrain = object;
		_terrain_toolbar.visible = false;
		_terrain_brush.visible = false;
	
func forward_spatial_gui_input(camera: Camera, event: InputEvent) -> bool:
	if _camera == null:
		_camera = camera;
		
	
	if _terrain == null:
		return false;
	
		
	var captured:bool = false;
	
	var ctrl_down:bool = Input.is_key_pressed(KEY_CONTROL) || Input.is_key_pressed(16777238) || Input.is_key_pressed(16777239);
	var shift_down:bool = Input.is_key_pressed(KEY_SHIFT);
	var mouse_middle_down:bool = Input.is_mouse_button_pressed(BUTTON_MIDDLE);
	var mouse_left_down:bool = Input.is_mouse_button_pressed(BUTTON_LEFT);
	var mouse_right_down:bool = Input.is_mouse_button_pressed(BUTTON_RIGHT);
	
	if event is InputEventMouseButton:
		
		if event.button_index == BUTTON_LEFT || event.button_index == BUTTON_RIGHT:
			captured = true;
			if event.pressed && self.draw_button_index == -1:
				self.draw_button_index = event.button_index;
				self.mouse_event("down", self.draw_button_index);
				self.mouse_event('draw', self.draw_button_index);
			elif !event.pressed && self.draw_button_index == event.button_index:
				self.mouse_event('released', self.draw_button_index);
			
		elif event.button_index == BUTTON_WHEEL_DOWN || event.button_index == BUTTON_WHEEL_UP || event.button_index == BUTTON_WHEEL_LEFT || event.button_index == BUTTON_WHEEL_RIGHT:
			if shift_down || ctrl_down || event.button_index == BUTTON_WHEEL_LEFT || event.button_index == BUTTON_WHEEL_RIGHT:
				captured = true;
			
			if !event.pressed:
				if event.button_index == BUTTON_WHEEL_DOWN && !shift_down && ctrl_down:
					_terrain_toolbar.get_node("RadiusLabel/RadiusSlider").value -= 1;
						
				if event.button_index == BUTTON_WHEEL_UP && !shift_down && ctrl_down:
					_terrain_toolbar.get_node("RadiusLabel/RadiusSlider").value += 1;
				
				if event.button_index == BUTTON_WHEEL_LEFT || (event.button_index == BUTTON_WHEEL_UP && shift_down):
					_terrain_toolbar.get_node("HeightLabel/HeightSlider").value += 1;
				
				if (event.button_index == BUTTON_WHEEL_RIGHT) || (event.button_index == BUTTON_WHEEL_DOWN && shift_down):
					_terrain_toolbar.get_node("HeightLabel/HeightSlider").value -= 1;
				
		
	if event is InputEventKey:
		
		if event.scancode == KEY_SPACE && !event.pressed:
			_terrain_toolbar.get_node("LockHeight").pressed = !_terrain_toolbar.get_node("LockHeight").pressed;
			_terrain_brush.is_height_locked = _terrain_toolbar.get_node("LockHeight").pressed;
			captured = true;
			
		if event.scancode == KEY_X && !event.pressed && !ctrl_down:
			_terrain_toolbar.swap_colors();
			_terrain_brush.swap_colors();
			captured = true;
		
		if event.scancode == KEY_S:
			var color:Color = _terrain.color(_terrain_brush.grid_position.x, _terrain_brush.grid_position.z);
			_terrain_brush.set_color(color);
			_terrain_toolbar.set_color(color);
			
		if event.pressed:
			if event.scancode == KEY_PLUS || event.scancode == 61:
				_terrain_toolbar.get_node("HeightLabel/HeightSlider").value += 1;
			if event.scancode == KEY_MINUS:
				_terrain_toolbar.get_node("HeightLabel/HeightSlider").value -= 1;
			
			if event.scancode == KEY_BRACELEFT || event.scancode == 123:
				_terrain_toolbar.get_node("RadiusLabel/RadiusSlider").value -= 1;
			if event.scancode == KEY_BRACERIGHT || event.scancode == 125:
				_terrain_toolbar.get_node("RadiusLabel/RadiusSlider").value += 1;
			
			
	
	if event is InputEventMouseMotion:
		var rayhit:Dictionary = self.raycast_mouse(event.position);
		if !rayhit.empty():
			if _terrain_brush.set_mouse_position(rayhit.position) && self.draw_button_index != -1:
				self.mouse_event("draw", self.draw_button_index);
				
		captured = true;
		if mouse_middle_down:
			captured = false;
	
	return captured;

func mouse_event(type, button_index):
	match type:
		"down":
			_terrain_brush.begin_stroke();
		"draw":
			if _terrain.editing_mode == TerrainNode.EditingMode.DRAW_TERRAIN:
				_terrain.draw_at(_terrain_brush.grid_position.x, _terrain_brush.grid_position.z, _terrain_brush.size, _terrain_brush.draw_mode, button_index, _terrain_brush.is_height_locked, _terrain_brush.locked_height);
				_terrain_brush.update_position();
			elif _terrain.editing_mode == TerrainNode.EditingMode.VERTEX_COLORS:
				_terrain.paint_vertex(_terrain_brush.grid_position, _terrain_brush.size.x, _terrain_brush.color0, _terrain_brush.opacity, _terrain_brush.blend_mode);
		"released":
			_terrain_brush.end_stroke();
			_terrain.end_stroke();
			self.draw_button_index = -1;
			if _terrain.editing_mode != TerrainNode.EditingMode.PREVIEW_PUBLISHED:
				_terrain_brush.update_position();
				var undo_redo = get_undo_redo()
				undo_redo.create_action("Terrain draw")
				undo_redo.add_do_method(self, "do_draw")
				undo_redo.add_undo_method(self, "undo_draw")
				undo_redo.commit_action();
			
func do_draw():
	if _terrain:
		_terrain.do_redo();
		
func undo_draw():
	if _terrain:
		_terrain.do_undo();
	
			
func raycast_mouse(mouse_pos:Vector2) -> Dictionary:
	if _camera && self.get_tree().get_edited_scene_root():
		var direct_space_state:PhysicsDirectSpaceState = self.get_tree().get_edited_scene_root().get_world().direct_space_state;
		var origin = _camera.project_ray_origin(mouse_pos);
		var dir = _camera.project_ray_normal(mouse_pos) * 1000.0;
		var rayHit:Dictionary = direct_space_state.intersect_ray(origin, origin + dir);
		return rayHit;
	return {};

func _on_terrain_property_changed (property:String, value):
	match property:
		"editing_mode":
			_terrain_brush.set_mode(_terrain.editing_mode);
			_terrain_toolbar.set_mode(_terrain.editing_mode);
	
func _on_menu_action (action:String, value):
	if _terrain:
		match action:
			"resize":
				_terrain.resize_canvas(value['chunk_size'], value['num_chunks'], value['x_coord'], value['z_coord']);
			"clear":
				_terrain.clear();
			"export":
				_terrain.export_mesh(value);
			
			
			#"autocolor":
			#	var new_color:Color = value['new_color'];
			#	var blend:int = value['blend'];
			#	var strength:float = value['blend_strength'];	
			#	match int(value['mode']):
			#		0:
			#			_terrain.color_by_slope(float(value['min_slope']), float(value['max_slope']), new_color, strength, blend);		
			#		1:
			#			_terrain.replace_color(Color(value['old_color']), float(value['similarity']), new_color, strength, blend);
			
