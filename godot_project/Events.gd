extends Node

signal mouse_event (event_type, grid_position, button_index)
signal property_change (propname, value)
signal exec_function (func_name, value)
signal query_map_size ()
signal return_map_size (num_chunks, chunk_size)

func connect_signals (object:Node, signal_names:Array):
	for signal_name in signal_names:
		var method_name = "_on_" + signal_name;
		var _has_signal:bool = self.has_signal(signal_name);
		var _has_method:bool = object.has_method(method_name);
		
		if !_has_signal:
			print("Signal '%s' not found" % signal_name);
			return;
		
		if !_has_method:
			print("No method '%s' found for %s" % [method_name, object.name]);
			return;
			
		self.connect(signal_name, object, method_name);


