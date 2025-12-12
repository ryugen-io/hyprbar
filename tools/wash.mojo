import sys
from subprocess import run

# Helper to get filename from path string manually since Path.name was flaky
fn get_filename(path: String) -> String:
    var p = path
    # Find last slash
    var idx = -1
    for i in range(len(p) - 1, -1, -1):
        if p[i] == "/":
            idx = i
            break
    
    if idx == -1:
        return p
    
    return p[idx+1:]

fn main() raises:
    var args = sys.argv()
    if len(args) < 2:
        print("Usage: mojo wash.mojo <plugin_source.rs>")
        return

    var source_path_str = String(args[1])
    var basename = get_filename(source_path_str)
    var dish_name = basename.replace(".rs", "").replace("examples/", "").replace("src/", "")
    
    # 1. Read source using open()
    var f = open(source_path_str, "r")
    var source_content = f.read()
    f.close()

    # 2. Parse Metadata
    var name = String("Unknown")
    var version = String("0.0.1")
    var author = String("")
    var description = String("")
    var extra_deps_str = String("")

    var lines = source_content.splitlines()
    for line in lines:
        var stripped = line.strip()
        if stripped.startswith("//!"):
            var content_str = stripped[3:].strip()
            if ":" in content_str:
                var parts = content_str.split(":", 1)
                var key = parts[0].strip().lower()
                var val = parts[1].strip()
                
                if key == "name":
                    name = String(val)
                elif key == "version":
                    version = String(val)
                elif key == "author":
                    author = String(val)
                elif key == "description":
                    description = String(val)
                elif key == "dependency":
                    extra_deps_str += String(val) + "\n"

    # Manual JSON construction
    var json_str = String('{"name": "')
    json_str += name + '", "version": "' + version + '", "author": "' + author + '", "description": "' + description + '"}'
    var escaped_json = json_str.replace('"', '\\"')

    # 3. Inject Metadata Shim
    var injection = String('\n#[unsafe(no_mangle)]\npub extern "C" fn _plugin_metadata() -> *const std::ffi::c_char {\n    static META: &[u8] = b"')
    injection += escaped_json + '\\0";\n    META.as_ptr() as *const _\n}\n'
    
    var final_source = source_content + injection

    # 4. Setup Temp Dir
    var cwd_str = run("pwd").strip()
    var temp_dir = String(cwd_str)
    temp_dir += "/.wash/" + dish_name
    
    # Clean and recreate
    _ = run(String("rm -rf ") + temp_dir)
    _ = run(String("mkdir -p ") + temp_dir + "/src")

    # 5. Write Cargo.toml
    var cargo_toml = String('[package]\nname = "')
    cargo_toml += dish_name + '"\nversion = "0.1.0"\nedition = "2024"\n\n[workspace]\n\n[lib]\ncrate-type = ["cdylib"]\n\n[dependencies]\nks-core = { path = "../../ks-core" }\nratatui = "0.29.0"\ntachyonfx = { version = "0.21.0", features = ["std-duration"] }\n'
    cargo_toml += extra_deps_str

    var f_toml = open(temp_dir + "/Cargo.toml", "w")
    f_toml.write(cargo_toml)
    f_toml.close()

    var f_src = open(temp_dir + "/src/lib.rs", "w")
    f_src.write(final_source)
    f_src.close()

    # 6. Build
    var msg = String("🧼 Washing ") + dish_name + "..."
    print(msg)
    
    var build_cmd = String("cd ")
    build_cmd += temp_dir + " && cargo build --release"
    var res = run(build_cmd)
    
    # Check output manually since we removed Path import to be safe? 
    # Actually we need to check existence. open() throws if not found? 
    # Let's trust ls output or try open 'r'
    
    var target_artifact = String(temp_dir)
    target_artifact += "/target/release/lib" + dish_name + ".so"
    
    var output_name = String(dish_name) 
    output_name += ".dish"
    
    # Basic existence check via run(ls) if we want to avoid exceptions or imports
    # But try/except open is idiomatic
    var exists = False
    try:
        var test_f = open(target_artifact, "r")
        test_f.close()
        exists = True
    except:
        exists = False
        
    if exists:
        _ = run(String("cp ") + target_artifact + " " + output_name)
        var done_msg = String("✨ Dish ready: ") + output_name
        print(done_msg)
    else:
        var err_msg = String("❌ Build failed or artifact not found: ") + target_artifact
        print(err_msg)
