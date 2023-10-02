source ~/.bashrc
cd godot_project

Godot --export "Windows Desktop" ../export/lil_leveler_windows.exe
sleep 15

# Build M1
sed -i -E 's/x86_64-apple-darwin/aarch64-apple-darwin/' terrain/library.gdnlib 
Godot --path godot_project --editor

sleep 30

# Build Intel
sed -i -E 's/aarch64-apple-darwin/x86_64-apple-darwin/' terrain/library.gdnlib 
Godot --path godot_project --editor
