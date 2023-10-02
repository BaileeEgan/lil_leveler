tool
extends Control

signal menu_action (action, value)

var num_chunks:int = 4;
var chunk_size:int = 16;

func _ready():
	for node in $ResizePopup/VBoxContainer/CenterContainer/GridContainer.get_children():
		node.connect("gui_input", self, "_on_resize_grid_input", [node]);
	$Autocolor/VBoxContainer/ReplaceColor.visible = false;
	$Autocolor/VBoxContainer/SlopeSelection.visible = true;

func load_map_params(chunk_size, num_chunks):
	self.num_chunks = num_chunks;
	self.chunk_size = chunk_size;
	$ResizePopup/VBoxContainer/VBoxContainer/ChunkSize/ChunkSizeSpinBox.value = chunk_size;
	$ResizePopup/VBoxContainer/VBoxContainer/NumChunks/NumChunksSpinBox.value = num_chunks;
	

func _on_show_popup (popup_name:String):
	match popup_name:
		"resize":
			self.set_resize_label("before", self.chunk_size, self.num_chunks);
			self.set_resize_label("after", self.chunk_size, self.num_chunks);
			$ResizePopup.popup();
		"export":
			$Export.popup();
		"clear":
			$Clear.popup();
		#"autocolor":
		#	$Autocolor.popup();
			
func _on_Clear_confirmed():
	$Clear.hide();
	self.emit_signal("menu_action", "clear", null);

func _on_Export_file_selected(path):
	$Export.hide();
	self.emit_signal("menu_action", "export", path);

func _on_ResizeCancel_button_up():
	$ResizePopup.hide();
	
func _on_resize_grid_input (event:InputEvent, node:ColorRect):
	if event is InputEventMouseButton && !event.pressed && event.button_index == BUTTON_LEFT:
		for child in $ResizePopup/VBoxContainer/CenterContainer/GridContainer.get_children():
			child.modulate = Color(0.462745, 0.462745, 0.462745);
		node.modulate = Color.white;

func set_resize_label (label_name:String, chunk_size, num_chunks):
	var map_size = chunk_size * num_chunks;
	var label =  str(map_size) + " x " + str(map_size);
	if label_name == "before":
		$ResizePopup/VBoxContainer/HBoxContainer2/BeforeSize.text = label;
	if label_name == "after":
		$ResizePopup/VBoxContainer/HBoxContainer2/AfterSize.text = label;

func _on_resize_value_changed(value, varname):
	var chunk_size:int = $ResizePopup/VBoxContainer/VBoxContainer/ChunkSize/ChunkSizeSpinBox.value;
	var num_chunks:int = $ResizePopup/VBoxContainer/VBoxContainer/NumChunks/NumChunksSpinBox.value;
	self.set_resize_label("after", chunk_size, num_chunks);

func _on_Resize_button_up():
	$ResizePopup.hide();
	var chunk_size:int = $ResizePopup/VBoxContainer/VBoxContainer/ChunkSize/ChunkSizeSpinBox.value;
	var num_chunks:int = $ResizePopup/VBoxContainer/VBoxContainer/NumChunks/NumChunksSpinBox.value;
	var idx:int = -1;
	for child in $ResizePopup/VBoxContainer/CenterContainer/GridContainer.get_children():
		if child.modulate == Color.white:
			idx = child.get_index();
	var x = idx % 3;
	var z = floor (idx / 3);
	var val:Dictionary = {
		"chunk_size": chunk_size,
		"num_chunks": num_chunks,
		"x_coord": x,
		"z_coord": z
	}
	self.num_chunks = num_chunks;
	self.chunk_size = chunk_size;
	self.emit_signal("menu_action", "resize", val);

"""
func _on_AutoColorMode_item_selected(index):
	if index == 0:
		$Autocolor/VBoxContainer/SlopeSelection.visible = true;
		$Autocolor/VBoxContainer/ReplaceColor.visible = false;
	elif index == 1:
		$Autocolor/VBoxContainer/SlopeSelection.visible = false;
		$Autocolor/VBoxContainer/ReplaceColor.visible = true;
		


func _on_SlopeSelectionMaxRange_value_changed(value):
	$Autocolor/VBoxContainer/SlopeSelection/MinRange.max_value = value;


func _on_SlopeSelectionMinRange_value_changed(value):
	$Autocolor/VBoxContainer/SlopeSelection/MaxRange.min_value = value;


func _on_AutocolorButton_button_up():
	var autocolor_data:Dictionary = {}
	autocolor_data['mode'] = $Autocolor/VBoxContainer/VariableSelection/OptionButton.selected;
	autocolor_data['new_color'] = $Autocolor/VBoxContainer/NewColor/NewColorPicker.color;
	autocolor_data['blend'] = $Autocolor/VBoxContainer/NewColor/BlendOption.selected;
	autocolor_data['blend_strength'] = float($Autocolor/VBoxContainer/NewColor/SpinBox.value) / 100.0;
	match $Autocolor/VBoxContainer/VariableSelection/OptionButton.selected:
		0:
			autocolor_data['min_slope'] = 1.0 - cos(deg2rad($Autocolor/VBoxContainer/SlopeSelection/MinRange.value));
			autocolor_data['max_slope'] = 1.0 - cos(deg2rad($Autocolor/VBoxContainer/SlopeSelection/MaxRange.value));
		1:
			autocolor_data['old_color'] = $Autocolor/VBoxContainer/ReplaceColor/OldColorPicker.color;
			autocolor_data['similarity'] = float($Autocolor/VBoxContainer/ReplaceColor/SpinBox.value) / 100.0;
	self.emit_signal("menu_action", "autocolor", autocolor_data);
"""

