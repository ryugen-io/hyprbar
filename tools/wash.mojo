from python import Python
import sys

fn main() raises:
    var args = sys.argv()
    if len(args) < 2:
        print("Usage: mojo wash.mojo <plugin_source.rs>")
        return

    var source_path = args[1]
    var dish_name = source_path.replace(".rs", "").replace("examples/", "").replace("src/", "")
    
    var builtins = Python.import_module("builtins")
    var os = Python.import_module("os")
    var json = Python.import_module("json")
    var shutil = Python.import_module("shutil")
    var subprocess = Python.import_module("subprocess")

    # 1. Read source (Keep as PythonObject for easy parsing)
    var f = builtins.open(source_path, "r")
    var source_content_py = f.read()
    f.close()

    # 2. Parse Metadata (Using Python methods)
    var metadata_dict = Python.dict()
    metadata_dict["name"] = "Unknown"
    metadata_dict["description"] = ""
    metadata_dict["author"] = ""
    metadata_dict["version"] = "0.0.1"

    # Collect plugin dependencies
    var extra_deps = Python.list()

    var lines = source_content_py.splitlines()
    for line in lines:
        var stripped = line.strip()
        if stripped.startswith("//!"):
            var content = stripped[3:].strip()
            if ":" in content:
                var parts = content.split(":", 1)
                var key = parts[0].strip().lower()
                var value = parts[1].strip()
                # Handle dependencies separately
                if key == "dependency":
                    extra_deps.append(value)
                else:
                    metadata_dict[key] = value

    var json_str = String(json.dumps(metadata_dict))
    var escaped_json = json_str.replace('"', '\\"')

    # 3. Inject Metadata Shim
    var injection = '\n#[unsafe(no_mangle)]\npub extern "C" fn _plugin_metadata() -> *const std::ffi::c_char {\n    static META: &[u8] = b"' + escaped_json + '\\0";\n    META.as_ptr() as *const _\n}\n'
    
    # Cast python object to Mojo String for concatenation
    var final_source = String(source_content_py) + injection

    # 4. Setup Temp Dir
    var cwd = String(os.getcwd())
    var temp_dir = cwd + "/.wash/" + dish_name
    
    if os.path.exists(temp_dir):
        shutil.rmtree(temp_dir)
    os.makedirs(temp_dir + "/src")

    # 5. Write Cargo.toml
    var cargo_toml = """[package]
name = \"""" + dish_name + """\"
version = "0.1.0"
edition = "2024"

# Break workspace inheritance
[workspace]

[lib]
crate-type = ["cdylib"]

[dependencies]
ks-core = { path = "../../ks-core" }
ratatui = "0.29.0"
tachyonfx = { version = "0.21.0", features = ["std-duration"] }
"""
    # Append plugin-specific dependencies
    for dep in extra_deps:
        cargo_toml = cargo_toml + String(dep) + "\n"

    var f_toml = builtins.open(temp_dir + "/Cargo.toml", "w")
    f_toml.write(cargo_toml)
    f_toml.close()

    var f_src = builtins.open(temp_dir + "/src/lib.rs", "w")
    f_src.write(final_source)
    f_src.close()

    # 6. Build
    print("🧼 Washing " + dish_name + "...")
    
    var cmd = Python.list()
    cmd.append("cargo")
    cmd.append("build")
    cmd.append("--release")

    var result = subprocess.run(cmd, cwd=temp_dir, capture_output=False)
    
    # Validating return code
    if result.returncode != 0:
        print("❌ Build failed!")
        return

    # 7. Copy Artifact
    var target_artifact = temp_dir + "/target/release/lib" + dish_name + ".so"
    var output_name = dish_name + ".dish"
    
    if os.path.exists(target_artifact):
        # Manual copy to avoid shutil argument issues
        var src_f = builtins.open(target_artifact, "rb")
        var data = src_f.read()
        src_f.close()
        
        var dst_f = builtins.open(output_name, "wb")
        dst_f.write(data)
        dst_f.close()
        
        print("✨ Dish ready: " + output_name)
    else:
        print("❌ Artifact not found: " + target_artifact)
