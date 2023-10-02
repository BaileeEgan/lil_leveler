extends Control

enum FontSize {
	SMALL = 16,
	MEDIUM = 24,
	LARGE = 32
}

var settings = {
	"camera_speed": 5,
	"mouse_sensitivity": 5,
	"invert_mouse_x": false,
	"invert_mouse_y": false,
	"font_size": FontSize.MEDIUM,
	"theme": 0,
}

var file_options = {
	"Open heightmap [ctrl+O]": "open_heightmap",
	"Save heightmap [ctrl+S]": "save",
	"Save as heightmap [ctrl+shift+S]": "save_as",
	"Export as mesh": "export_mesh"
}

var edit_options = {
	"Resize canvas": "resize_canvas",
	"Clear all": "clear_all",
	"Themes": "show_themes",
	"Settings": "show_settings"
}

var active_menu_button:MenuButton = null;
var cursor_active:bool = true;
var current_mode:int = 0;
var current_file_path:String = "";


func _ready():
	
	Events.connect_signals(self,["property_change", "exec_function"]);
	
	var docs_dir = OS.get_system_dir(OS.SYSTEM_DIR_DOCUMENTS);
	$Import.current_dir = docs_dir;
	$Import.current_path = docs_dir + "/";
	$Save.current_dir = docs_dir;
	$Save.current_path = docs_dir + "/heights.save";
	$Export.current_dir = docs_dir;
	$Export.current_path = docs_dir + "/generated_terrain.obj"
	
	var idx = 0;
	for opt in file_options:
		$Panel/HBoxContainer/File.get_popup().add_item(opt, idx);
		idx += 1;
	idx = 0;
	for opt in edit_options:
		$Panel/HBoxContainer/Edit.get_popup().add_item(opt, idx);
		idx += 1;
	
	idx = 0;
	for node in $ToolPanel/VBoxContainer.get_children():
		if node is TextureButton:
			if idx != 0:
				node.get_child(0).visible = false;
				node.modulate = Color.darkgray;
			node.connect("mouse_entered", self, "_on_mode_button_mouse_entered", [node, idx]);
			node.connect("mouse_exited", self, "_on_mode_button_mouse_exited", [node, idx]);
			node.connect("button_up", self, "_on_mode_button_mouse_up", [node, idx]);
			idx += 1;

	for node in $Panel/HBoxContainer.get_children():
		if node is MenuButton:
			var popup:PopupMenu = node.get_popup();
			popup.connect("id_pressed", self, "_on_menu_item_clicked", [node]);

	idx = 0;
	for node in $ResizePanel/CenterContainer2/VBoxContainer/CenterContainer/GridContainer.get_children():
		node.connect("gui_input", self, "_on_resize_grid_input", [node, idx]);
	$ResizePanel.visible = false;
	$ThemePanel.visible = false;
	self.update_undo_redo(false, false);
	yield(get_tree(), "idle_frame");
	$ThemePanel/VBoxContainer/CenterContainer/ItemList.select(0);

func add_theme (theme):
	var item_list = $ThemePanel/VBoxContainer/CenterContainer/ItemList;
	item_list.add_item(theme);

func highlight_tool (index:int):
	var button = $ToolPanel/VBoxContainer.get_child(index);
	button.modulate = Color.white;
	button.get_child(0).visible = true;
	
func dehighlight_tool (index:int):
	var button = $ToolPanel/VBoxContainer.get_child(index);
	button.modulate = Color.darkgray;
	button.get_child(0).visible = false;
	
func update_undo_redo (can_undo:bool, can_redo):
	if can_undo:
		$Panel/HBoxContainer/Undo.disabled = false;
		$Panel/HBoxContainer/Undo.modulate = Color.white;
	else:
		$Panel/HBoxContainer/Undo.modulate = Color.darkgray;
		$Panel/HBoxContainer/Undo.disabled = true;
	
	if can_redo:
		$Panel/HBoxContainer/Redo.disabled = false;
		$Panel/HBoxContainer/Redo.modulate = Color.white;
	else:
		$Panel/HBoxContainer/Redo.modulate = Color.darkgray;
		$Panel/HBoxContainer/Redo.disabled = true;

func select_tool (index:int):
	if index <= TerrainTool.DrawMode.SMOOTH && index != self.current_mode:
		self.dehighlight_tool(self.current_mode);
		self.current_mode = index;
		self.highlight_tool(self.current_mode);
		Events.emit_signal("property_change", "draw_mode", index);

func set_map_params (num_chunks:int, chunk_size:int):
	$ResizePanel/CenterContainer2/VBoxContainer/VBoxContainer/NumChunks/NumChunksSpinBox.value = num_chunks;
	$ResizePanel/CenterContainer2/VBoxContainer/VBoxContainer/ChunkSize/ChunkSizeSpinBox.value = chunk_size;
	self.set_resize_label("before", num_chunks * chunk_size);
	self.set_resize_label("after", num_chunks * chunk_size);

func load_settings (_settings:Dictionary):
	for key in _settings:
		self.set_setting(key, _settings[key], true);


func set_resize_label (name, map_size:int):
	if name == "before":
		$ResizePanel/CenterContainer2/VBoxContainer/HBoxContainer2/BeforeSize.text = "%sx%s" % [map_size, map_size]; 	
	elif name == "after":
		$ResizePanel/CenterContainer2/VBoxContainer/HBoxContainer2/AfterSize.text = "%sx%s" % [map_size, map_size]; 	

func open_heightmap():
	$Import.popup();
	Events.emit_signal("property_change", "paused", true);

func save ():
	if self.current_file_path != "":
		Events.emit_signal("exec_function", "save_file", self.current_file_path);
	else:
		self.save_as();

func save_as():
	if self.current_file_path != "":
		$Save.current_dir = self.current_file_path.get_base_dir();
		$Save.current_file = self.current_file_path.get_file();
		$Save.current_path = self.current_file_path;
	$Save.popup();
	Events.emit_signal("property_change", "paused", true);

func export_mesh():
	if self.current_file_path != "":
		$Export.current_dir = self.current_file_path.get_base_dir();
		$Export.current_path = self.current_file_path.get_basename() + ".obj";
		$Export.current_file = $Export.current_path.get_file();
	$Export.popup();
	Events.emit_signal("property_change", "paused", true);
		
func resize_canvas():
	Events.call_deferred("emit_signal", "query_map_size");
	var args = yield(Events, "return_map_size");
	var num_chunks = args[0];
	var chunk_size = args[1];
	self.set_map_params(num_chunks, chunk_size);
	$ResizePanel.visible = true;
	Events.emit_signal("property_change", "paused", true);
	$ResizePanel/CenterContainer2/VBoxContainer/HBoxContainer/ResizeButton.visible = false;

func clear_all():
	Events.emit_signal("exec_function", "clear_all", true);

func show_themes():
	$ThemePanel.visible = true;

func show_settings():
	$Settings.popup();
	Events.emit_signal("property_change", "paused", true);
	pass
	
func set_setting (setting_name:String, value, update_ui:bool=true):
	if !self.settings.has(setting_name) || (self.settings.has(setting_name) && self.settings[setting_name] != value):
		self.settings[setting_name] = value;
		match setting_name:
			"font_size":
				var theme:Theme = self.theme;
				var font:DynamicFont = theme.default_font;
				font.size = value;
				$Panel.rect_min_size.y = value * 2 + 8;
				$Panel.rect_size.y = value * 2 + 8;
			"theme":
				Events.emit_signal("exec_function", "load_theme", value);
		Events.emit_signal("property_change", setting_name, value);
	
		if update_ui:
			match setting_name:
				"font_size":
					match value:
						FontSize.SMALL:
							$Settings/Panel/VBoxContainer/FontSizeOption.selected = 0;
						FontSize.MEDIUM:
							$Settings/Panel/VBoxContainer/FontSizeOption.selected = 1;
						FontSize.LARGE:
							$Settings/Panel/VBoxContainer/FontSizeOption.selected = 2;
				"camera_speed":
					$Settings/Panel/VBoxContainer/CamSpeedSlider.value = value;
				"mouse_sensitivity":
					$Settings/Panel/VBoxContainer/MouseSensSlider.value = value;
				"invert_mouse_x":
					$Settings/Panel/VBoxContainer/InvertMouseXCheck.pressed = value;
				"invert_mouse_y":
					$Settings/Panel/VBoxContainer/InvertMouseYCheck.pressed = value;
				"theme":
					$ThemePanel/VBoxContainer/CenterContainer/ItemList.select(value);

func _on_property_change (propname:String, value):
	pass

func _on_exec_function (funcname:String, value):
	if funcname == "cursor_resize":
		$Panel/HBoxContainer/RadiusSlider.value += value.x;
		$Panel/HBoxContainer/HeightSlider.value += value.y;
		
func _input(event):
	
	if event is InputEventKey:
		
		var ctrl_down =  Input.is_action_pressed("control") || Input.is_action_pressed("command");
		var shift_down = Input.is_action_pressed("shift");
		
		if ctrl_down && event.scancode == KEY_O && !event.pressed:
			self.open_heightmap();
	
		if ctrl_down && event.scancode == KEY_S && !event.pressed:
			if !shift_down:
				self.save();
			else:
				self.save_as();
		
		if !event.pressed && event.scancode >= 49 && event.scancode <= 54:
			var index = event.scancode - 49;
			self.select_tool(index);
		
		if !event.pressed && (event.scancode == KEY_SPACE || event.scancode == 32):
			$Panel/HBoxContainer/LockHeight.pressed = !$Panel/HBoxContainer/LockHeight.pressed;
			Events.emit_signal("property_change", "is_height_locked", $Panel/HBoxContainer/LockHeight.pressed);
		
		
	
	if event.is_action_pressed("shift"):
		$VBoxContainer/MiddleMouse/Label.text = "Pan"
		$VBoxContainer/WheelUp/Label.text = "Inc. brush height"
		$VBoxContainer/WheelDown/Label.text = "Dec. brush height"
	elif event.is_action_pressed("control"):
		$VBoxContainer/MiddleMouse/Label.text = "Rotate"
		$VBoxContainer/WheelUp/Label.text = "Inc. brush radius"
		$VBoxContainer/WheelDown/Label.text = "Dec. brush radius"
		
	elif event.is_action_released("shift") || event.is_action_released("control"):
		$VBoxContainer/MiddleMouse/Label.text = "Rotate"
		$VBoxContainer/WheelUp/Label.text = "Move up"
		$VBoxContainer/WheelDown/Label.text = "Move down"
	
	if event is InputEventMouseMotion:
		if event.position.y < $Panel.rect_size.y * 1.5 || event.position.x < 1.25 * ($ToolPanel.rect_size.x + $ToolPanel.rect_position.x) || $ResizePanel.visible || $Import.visible || $Save.visible || $Export.visible || $ThemePanel.visible:
			Events.emit_signal("property_change", "cursor_active", false);
		else:
			if self.active_menu_button != null:
				if self.active_menu_button.get_popup().visible:
					Events.emit_signal("property_change", "cursor_active", false);
				else:
					Events.emit_signal("property_change", "cursor_active", true);
			else:
				Events.emit_signal("property_change", "cursor_active", true);
				
func _on_File_about_to_show():
	self.active_menu_button = $Panel/HBoxContainer/File;

func _on_Edit_about_to_show():
	self.active_menu_button = $Panel/HBoxContainer/Edit;

func _on_mode_button_mouse_entered(button:TextureButton, index:int):
	self.highlight_tool(index);

func _on_mode_button_mouse_exited(button:TextureButton, index:int):
	if index != self.current_mode:
		self.dehighlight_tool(index);
	
func _on_mode_button_mouse_up(button:TextureButton, index:int):
	self.select_tool(index);
	

func _on_menu_item_clicked (id:int, button:MenuButton):
	var popup = button.get_popup();
	var text = popup.get_item_text(id);
	
	if self.file_options.has(text):
		self.call(self.file_options[text]);
	elif self.edit_options.has(text):
		self.call(self.edit_options[text]);
	
func _on_resize_grid_input (event:InputEvent, node:ColorRect, index:int):
	if event is InputEventMouseButton && !event.pressed && event.button_index == BUTTON_LEFT:
		for child in $ResizePanel/CenterContainer2/VBoxContainer/CenterContainer/GridContainer.get_children():
			child.modulate = Color(0.462745, 0.462745, 0.462745);
		node.modulate = Color.white;


func _on_ResizeButton_button_up():
	Events.emit_signal("property_change", "paused", false);
	$ResizePanel.visible = false;
	
	var num_chunks:int = $ResizePanel/CenterContainer2/VBoxContainer/VBoxContainer/NumChunks/NumChunksSpinBox.value;
	var chunk_size:int = $ResizePanel/CenterContainer2/VBoxContainer/VBoxContainer/ChunkSize/ChunkSizeSpinBox.value;
	var idx:int = -1;
	for child in $ResizePanel/CenterContainer2/VBoxContainer/CenterContainer/GridContainer.get_children():
		if child.modulate == Color.white:
			idx = child.get_index();
	
	var x = idx % 3;
	var y = floor (idx / 3);
	Events.emit_signal("exec_function", "resize_terrain", Rect2(chunk_size, num_chunks, x, y));

func _on_CancelResize_button_up():
	Events.emit_signal("property_change", "paused", false);
	$ResizePanel.visible = false;
	
func _on_Import_file_selected(path):
	Events.emit_signal("exec_function", "open_file", path);
	
func _on_Save_file_selected(path):
	Events.emit_signal("exec_function", "save_file", path);

func _on_Export_file_selected(path):
	Events.emit_signal("exec_function", "export_mesh", path);


func _on_StepHeightValue_value_changed(value):
	$Panel/HBoxContainer/StepHeightValue.get_line_edit().focus_mode = 0;
	Events.emit_signal("property_change", "step_height", float(value) / 100.0);

func _on_ChunkSizeSpinBox_value_changed(value):
	var num_chunks:int = $ResizePanel/CenterContainer2/VBoxContainer/VBoxContainer/NumChunks/NumChunksSpinBox.value;
	self.set_resize_label("after",num_chunks * value);
	$ResizePanel/CenterContainer2/VBoxContainer/HBoxContainer/ResizeButton.visible = true;
	
func _on_NumChunksSpinBox_value_changed(value):
	var chunk_size:int = $ResizePanel/CenterContainer2/VBoxContainer/VBoxContainer/ChunkSize/ChunkSizeSpinBox.value;
	self.set_resize_label("after",chunk_size * value);
	$ResizePanel/CenterContainer2/VBoxContainer/HBoxContainer/ResizeButton.visible = true;
	
func _on_Undo_button_up():
	Events.emit_signal("exec_function", "undo_redo", "undo");

func _on_Redo_button_up():
	Events.emit_signal("exec_function", "undo_redo", "redo");

func _on_LockHeight_pressed():
	Events.emit_signal("property_change", "is_height_locked", $Panel/HBoxContainer/LockHeight.pressed);


func _on_RadiusSlider_value_changed(value):
	$Panel/HBoxContainer/RadiusLabel.text = "[" + str(value) + "]"
	Events.emit_signal("property_change", "cursor_width", value);

func _on_HeightSlider_value_changed(value):
	$Panel/HBoxContainer/HeightLabel.text = "[" + str(value) + "]"
	Events.emit_signal("property_change", "cursor_height", value);

func _on_SmoothnessSlider_value_changed(value):
	$Panel/HBoxContainer/SmoothnessLabel.text = "[" + str(value) + "]"
	Events.emit_signal("property_change", "cursor_smoothness", float(value) / 10.0);

func _on_CloseThemes_button_up():
	$ThemePanel.visible = false;

func _on_popup_hide():
	Events.emit_signal("property_change", "paused", false);

func _on_Close_button_up():
	$Settings.hide();

func _on_setting_changed(value, setting_name):
	if setting_name != "font_size":
		self.set_setting(setting_name, value, false);
	else:
		match value:
			0:
				self.set_setting("font_size", FontSize.SMALL);
			1:
				self.set_setting("font_size", FontSize.MEDIUM);
			2:
				self.set_setting("font_size", FontSize.LARGE);
