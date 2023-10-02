extends Spatial


var paused:bool = false;

var move_speed:float = 1.0;
var mouse_look_direction:Vector2 = Vector2.ONE;

export (float, 1, 30) var distance:float = 10.0 setget set_distance;
export var min_distance:float = 2.0;
export var max_distance:float = 30.0;

export (float, -90, 90) var min_pitch_angle:float = 10;
export (float, -90, 90) var max_pitch_angle:float = 85;
export (float, 0, 360) var max_yaw:float = 360;

# Interpolation speed of camera to target position
export (float, 0.0, 1.0) var camera_smoothness:float = 0.5;

# Interpolation speed of mouse look
export (float, 0.0, 1.0) var camera_look_speed:float = 0.5;


export (float, 0, 360) var pitch:float = 0;
export (float, 0, 360) var yaw:float = 0;

export (NodePath) var following_target:NodePath;

export var pitch_angle = 0.0;

var target_distance:float;

var mouse_grid_position:Vector3;
var mouse_world_position:Vector3;
var last_screen_position:Vector2;

var cursor_active:bool = true;

func _ready():
	Events.connect_signals(self,["property_change"]);
	self.target_distance = self.distance;
	$Camera.current = self.visible;

func set_distance(val:float):
	val = clamp(val, min_distance, max_distance);
	distance = val;
	if has_node("Camera"):
		var camera:Camera = get_node("Camera");
		camera.transform.origin = camera.transform.basis.z * distance;
	return distance;
	



var input_directions = {
	"forward": Vector3.FORWARD,
	"back": Vector3.BACK,
	"right": Vector3.RIGHT,
	"left": Vector3.LEFT,
	"up": Vector3.UP,
	"down": Vector3.DOWN,
}

const NO_Y = Vector3(1, 0, 1);
var velocity:Vector3 = Vector3.ZERO;
export var speed:float = 100;

func _process(delta):
	if Engine.editor_hint || delta == 0:
		return
		
	var lerp_step:float = smoothstep(15, 120, 1.0 / delta);
	mouse_position *= lerp_step * 0.4 + 0.5;
	self.distance = lerp(self.distance, self.target_distance, lerp_step * 0.1 + 0.1);
	lerp_step = (1.0 - lerp_step * 0.4 - 0.1);
	self.pitch = clamp(lerp(self.pitch, self.pitch + mouse_position.y, lerp_step), min_pitch_angle, max_pitch_angle);
	self.yaw = wrapf(lerp(self.yaw, self.yaw + mouse_position.x, lerp_step), 0, 360);

	self.rotation_degrees.x = -pitch;
	self.rotation_degrees.y = -yaw;
	
	var key_down:bool = false;
	for key in self.input_directions:
		if Input.is_action_pressed("move_" + key) && !Input.is_action_pressed("command") && !Input.is_action_pressed("control"):
			self.velocity += self.input_directions[key];
			key_down = true;
	if key_down:
		self.velocity = self.velocity.normalized()
	
	var movement_direction:Vector3 = Vector3.ZERO;
	movement_direction += self.move_speed * ($Camera.global_transform.basis.z * NO_Y).normalized() * self.velocity.z;
	movement_direction += self.move_speed * ($Camera.global_transform.basis.x * NO_Y).normalized() * self.velocity.x;
	movement_direction += self.move_speed * Vector3.UP * self.velocity.y;
	
	self.velocity *= 0.5;
	self.global_transform.origin += movement_direction * delta * self.speed;
	
	
var draw_down = false;
var draw_index = -1;
var middle_down = false
var shift_down = false;
var mouse_position:Vector2;


func _input(event):
	if self.paused:
		return;
		
	var ctrl_down =  Input.is_action_pressed("control") || Input.is_action_pressed("command");
	var shift_down = Input.is_action_pressed("shift");
		
	if Input.is_action_just_released("ui_cancel"):
		if Input.get_mouse_mode() == Input.MOUSE_MODE_HIDDEN:
			Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE);
		else:
			Input.set_mouse_mode(Input.MOUSE_MODE_HIDDEN);
			
			
	if event is InputEventKey:
		if event.pressed:
			if event.scancode == 123:
				Events.emit_signal("exec_function", "cursor_resize", Vector2(-1, 0))
			if event.scancode == 125:
				Events.emit_signal("exec_function", "cursor_resize", Vector2(1, 0));
			if event.scancode == 45:
				Events.emit_signal("exec_function", "cursor_resize", Vector2(0, -1));
			if event.scancode == 61:
				Events.emit_signal("exec_function", "cursor_resize", Vector2(0, 1));
		else:
			if event.scancode == KEY_Z:
				if ctrl_down:
					if shift_down:
						Events.emit_signal("exec_function", "undo_redo", "redo");
					else:
						Events.emit_signal("exec_function", "undo_redo", "undo");
						
			elif event.scancode == KEY_Y:
				if ctrl_down:
					Events.emit_signal("exec_function", "undo_redo", "redo");
			
			
	
	if event is InputEventMouseMotion:
		if middle_down:
			
			if shift_down:
				self.velocity.z = -event.relative.y * 0.05;
				self.velocity.x = -event.relative.x * 0.05;
			else:
				mouse_position = mouse_position.linear_interpolate(event.relative * 2.0 * self.camera_look_speed * self.mouse_look_direction, 0.5);
		

		else:
			if last_screen_position.distance_squared_to(event.position) > 20.0:
				self.last_screen_position = event.position;
				var raycast:Dictionary = self.raycast_mouse(event.position);
				if !raycast.empty():
					self.mouse_world_position = raycast.position;
					var new_mouse_grid_position = (self.mouse_world_position).round();
					new_mouse_grid_position.y = 0;
					if !new_mouse_grid_position.is_equal_approx(self.mouse_grid_position):
						self.mouse_grid_position = new_mouse_grid_position;
						Events.emit_signal("mouse_event", "move", self.mouse_grid_position);
						if self.draw_down && self.draw_index != -1:
							Events.emit_signal("mouse_event", "draw", self.mouse_grid_position, self.draw_index);
				
	if event is InputEventMouseButton:
		if event.button_index == BUTTON_MIDDLE:
			self.middle_down = event.pressed
		
		elif event.button_index == BUTTON_LEFT || event.button_index == BUTTON_RIGHT:
			if !self.draw_down && event.pressed && self.draw_index == -1:
				self.draw_index = event.button_index
				Events.emit_signal("mouse_event", "draw", self.mouse_grid_position, event.button_index);
			elif self.draw_down && !event.pressed:
				Events.emit_signal("mouse_event", "release",  self.mouse_grid_position);
				self.draw_index = -1;
			self.draw_down = event.pressed;
		
		
		if self.cursor_active && event.pressed:
			if event.button_index == BUTTON_WHEEL_DOWN && !shift_down:
				if !ctrl_down:
					self.velocity.y += 1;
				else:
					Events.emit_signal("exec_function", "cursor_resize", Vector2(-1, 0));
					
			elif event.button_index == BUTTON_WHEEL_UP && !shift_down:
				if !ctrl_down:
					self.velocity.y -= 1;
				else:
					Events.emit_signal("exec_function", "cursor_resize", Vector2(1, 0));
			
			elif (event.button_index == BUTTON_WHEEL_LEFT) || (event.button_index == BUTTON_WHEEL_UP && shift_down):
				Events.emit_signal("exec_function", "cursor_resize", Vector2(0, 1));
			elif (event.button_index == BUTTON_WHEEL_RIGHT) || (event.button_index == BUTTON_WHEEL_DOWN && shift_down):
				Events.emit_signal("exec_function", "cursor_resize", Vector2(0, -1));
					
	
func raycast_mouse(mouse_pos:Vector2) -> Dictionary:
	if $Camera:
		var direct_space_state:PhysicsDirectSpaceState = self.get_world().direct_space_state;
		var origin =$Camera.project_ray_origin(mouse_pos);
		var dir = $Camera.project_ray_normal(mouse_pos) * 200.0;
		var rayHit:Dictionary = direct_space_state.intersect_ray(origin, origin + dir);
		return rayHit;
	return {};

func _on_property_change (propname:String, value):
	if propname == "cursor_active":
		self.cursor_active = value;
	if propname == "paused":
		self.paused = value;
		self.set_process(!value);
	if propname == "invert_mouse_x":
		if value:
			self.mouse_look_direction.x = -1;
		else:
			self.mouse_look_direction.x = 1;
	if propname == "invert_mouse_y":
		if value:
			self.mouse_look_direction.y = -1;
		else:
			self.mouse_look_direction.y = 1;
	if propname == "camera_speed":
		self.move_speed = float(value) / 5.0;
	if propname == "mouse_sensitivity":
		self.camera_look_speed = float(value) / 10.0;
		
