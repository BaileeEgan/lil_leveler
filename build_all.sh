for target in x86_64-apple-darwin x86_64-pc-windows-gnu aarch64-apple-darwin; do
	cargo build --target $target --target-dir plugin/addons/lil_leveler/native
	#cargo build --target $target --release --target-dir plugin/addons/lil_leveler/native
	#cp -r target/"$target"/release/* plugin/addons/lil_leveler/native/"$target"/release
	#cp -r target/"$target"/debug/* plugin/addons/lil_leveler/native/"$target"/debug
done
