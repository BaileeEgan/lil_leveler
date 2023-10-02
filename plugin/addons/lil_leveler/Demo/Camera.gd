extends Spatial

export var following:NodePath;

func _ready():
	pass

func _process(_delta):
	if !following.is_empty():
		var target = self.get_node(following);
		self.global_transform.origin = self.global_transform.origin.linear_interpolate(target.global_transform.origin, 0.5);

func _input (event:InputEvent):
	if event is InputEventMouseMotion:
		self.rotation_degrees.y -= event.relative.x * 0.1;
		self.rotation_degrees.x -= event.relative.y * 0.1;

