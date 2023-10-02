tool
class_name TerrainToolbar
extends HBoxContainer

signal brush_changed (property, value)
signal show_popup (popup_name)

var current_draw_mode:int = 0;
const menu_options = [
	{ "popup": "resize", "text": "Resize canvas", },
	{ "popup": "clear", "text": "Clear all", },
	{ "popup": "export", "text": "Export as .OBJ", }
]

func _ready():
	for i in range($DrawModeSelection.get_child_count()):
		var button:TextureButton = $DrawModeSelection.get_child(i);
		button.connect("button_up", self, "_on_draw_mode_selected", [i]);
		if i > 0:
			button.modulate = Color.dimgray;
	var popup:PopupMenu = $MenuButton.get_popup();
	popup.connect("id_pressed", self, "_on_menu_selected");
	
func set_mode (mode:int):
	match mode:
		TerrainNode.EditingMode.DRAW_TERRAIN:
			self.visible = true;
			$DrawModeSelection.visible = true;
			$ColorSelection1.visible = false;
			$ColorSelection2.visible = false;
			$BlendOptions.visible = false;
			$LockHeight.visible = true;
			
			$HeightLabel.visible = true;
			
			$OpacityLabel.visible = false;
			
		TerrainNode.EditingMode.VERTEX_COLORS:
			self.visible = true;
			$DrawModeSelection.visible = false;
			$ColorSelection1.visible = true;
			$ColorSelection2.visible = true;
			$BlendOptions.visible = true;
			$LockHeight.visible = false;
			
			$HeightLabel.visible = false;
			
			$OpacityLabel.visible = true;
			
		TerrainNode.EditingMode.PREVIEW_PUBLISHED:
			self.visible = false;
			
func swap_colors():
	var tmp = $ColorSelection2.color;
	$ColorSelection2.color = $ColorSelection1.color;
	$ColorSelection1.color = tmp;

func set_color(color:Color):
	$ColorSelection1.color = color;
		
			
func _on_menu_selected (index:int):
	self.emit_signal("show_popup", self.menu_options[index]['popup']);

func _on_menu_action (action:String, value):
	pass
	
func _on_draw_mode_selected(button_index:int):
	if button_index != self.current_draw_mode:
		self.current_draw_mode = button_index;
		for i in range($DrawModeSelection.get_child_count()):
			var button:TextureButton = $DrawModeSelection.get_child(i);
			if i != self.current_draw_mode:
				button.modulate = Color.dimgray;
		$DrawModeSelection.get_child(button_index).modulate = Color.white;
		self.emit_signal("brush_changed", "draw_mode", button_index);


func _on_brush_size_changed(value, property_name:String):
	self.emit_signal("brush_changed", property_name, value);
	if property_name == "radius":
		$RadiusLabel/HBoxContainer/Value.text = "[" + str(value) + "]"
	elif property_name == "height":
		$HeightLabel/HBoxContainer/Value.text = "[" + str(value) + "]"
	elif property_name == "opacity":
		$OpacityLabel/HBoxContainer/Value.text = "[" + str(value) + "]"
	
func _on_LockHeight_button_up():
	self.emit_signal("brush_changed", "lock_height", $LockHeight.pressed);


func _on_BlendOptions_item_selected(index):
	self.emit_signal("brush_changed", "blend_mode", index);

func _on_ColorSelection_color_changed(color, index):
	self.emit_signal("brush_changed", "color" + str(index), color);
