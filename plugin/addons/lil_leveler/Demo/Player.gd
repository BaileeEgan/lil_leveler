extends KinematicBody

var speed = 10.0;
var target_look_at_position:Vector3;
var velocity:Vector3 = Vector3.ZERO;

const NO_Y = Vector3(1, 0, 1);

func _ready():
	self.target_look_at_position = self.global_transform.origin - self.global_transform.basis.z * 100.0;
	
func _process(delta):

	if Input.is_key_pressed(KEY_W):
		self.velocity.z = -1;
		self.target_look_at_position = self.global_transform.origin - get_viewport().get_camera().global_transform.basis.z * 100.0 * NO_Y;
		
	if Input.is_key_pressed(KEY_S):
		self.velocity.z = -1;
		self.target_look_at_position = self.global_transform.origin +  get_viewport().get_camera().global_transform.basis.z * 100.0* NO_Y;
		
	if Input.is_key_pressed(KEY_A):
		self.velocity.z = -1;
		self.target_look_at_position = self.global_transform.origin -  get_viewport().get_camera().global_transform.basis.x * 100.0* NO_Y;
		
	if Input.is_key_pressed(KEY_D):
		self.velocity.z = -1;
		self.target_look_at_position = self.global_transform.origin +  get_viewport().get_camera().global_transform.basis.x * 100.0* NO_Y;
		
	self.velocity.x *= 0.8;
	self.velocity.z *= 0.8;
	
	if !self.target_look_at_position.is_equal_approx(self.global_transform.origin) && self.target_look_at_position.distance_squared_to(self.global_transform.origin) > 0.1:
		var look_at = self.global_transform.looking_at(self.target_look_at_position, Vector3.UP);
		self.global_transform.basis = self.global_transform.basis.slerp(look_at.basis, 0.1);
	
	
	
func _physics_process(delta):
	if !self.is_on_floor():
		self.velocity.y -= 1 * delta * 5.0;
	else:
		self.velocity.y = 0;
	
	var movement:Vector3 = Vector3.ZERO;
	movement += self.velocity.z * self.global_transform.basis.z;
	movement += self.velocity.x * self.global_transform.basis.x;
	movement *= self.speed;
	movement += self.velocity.y * Vector3.UP * 10.0;
	self.move_and_slide(movement, Vector3.UP);
	
	


