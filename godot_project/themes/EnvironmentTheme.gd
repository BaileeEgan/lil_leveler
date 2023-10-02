tool
class_name EnvironmentTheme
extends Resource

export var background_ambient_color = Color(0.023529, 0, 0.247059);
export var background_sky_color = Color(0.647059, 0.839216, 0.945098);
export var background_horizon_color = Color(0.839216, 0.917647, 0.980392);
export var light_color = Color.white;

export var mesh_main_color:Color = Color(0.501961, 0.501961, 0.501961);
export var mesh_grid_color:Color = Color.white;
export var mesh_grid_width:float = 1.0;
export var mesh_show_grid:bool = true;

export var cursor_top_color:Color = Color.red;
export var cursor_bottom_color:Color = Color(0.25, 0.0, 0.25);

export var environment:Environment = null;

export var do_load_from_env:bool = false setget _load_from_env;
func _load_from_env (val:bool):
	if val && environment:
		load_from_environment(environment);
	return false;

func load_from_environment (env:Environment):
	self.background_ambient_color = env.ambient_light_color;
	self.background_sky_color = (env.background_sky as ProceduralSky).sky_top_color;
	self.background_horizon_color = (env.background_sky as ProceduralSky).sky_horizon_color;


export var do_generate_env:bool = false setget _generate_env;
func _generate_env (val:bool):
	if val:
		generate_environment();
	return false;
	
func generate_environment () -> Environment:
	var env:Environment;
	if self.environment == null:
		env = Environment.new();
	else:
		env = self.environment;
	
	var sky:ProceduralSky;
	if env.background_sky == null:
		sky = ProceduralSky.new();
	else:
		sky = env.background_sky;
	
	env.background_mode = Environment.BG_SKY;
	sky.sky_curve = 0.5
	sky.ground_curve = 0.5
	sky.sky_top_color = self.background_sky_color;
	sky.ground_bottom_color = self.background_sky_color;
	sky.sky_horizon_color = self.background_horizon_color;
	sky.ground_horizon_color= self.background_horizon_color;
	sky.sun_curve = 0.0025;
	sky.texture_size = ProceduralSky.TEXTURE_SIZE_512;
	sky.sun_angle_max = 0;
	sky.sun_angle_min = 0;
	env.ambient_light_sky_contribution = 0.125;
	env.background_sky = sky;
	env.ambient_light_color = self.background_ambient_color;
	self.environment = env;
	return env;
