use include_dir::Dir;

fn main() {
	println!("cargo:rerun-if-changed=frontend/dist");
}

const _: Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");
