import sys
from subprocess import run

# ANSI Colors
fn get_color_reset() -> String: return "\033[0m"
fn get_color_red() -> String: return "\033[31m"
fn get_color_green() -> String: return "\033[32m"
fn get_color_blue() -> String: return "\033[34m"
fn get_color_yellow() -> String: return "\033[33m"
fn get_color_cyan() -> String: return "\033[36m"
fn get_color_magenta() -> String: return "\033[35m"

fn get_icon_wash() -> String: return get_color_cyan() + "" + get_color_reset()
fn get_icon_load() -> String: return get_color_magenta() + "" + get_color_reset()
fn get_icon_check() -> String: return get_color_green() + "" + get_color_reset()
fn get_icon_error() -> String: return get_color_red() + "" + get_color_reset()
fn get_icon_warn() -> String: return get_color_yellow() + "" + get_color_reset()

# Helper to check directory existence via ls -d
fn dir_exists(path: String) -> Bool:
    try:
        var cmd = String("ls -d ") + path
        _ = run(cmd)
        return True
    except:
        return False

# Helper to check file existence via ls (Pure Mojo workaround)
fn file_exists(path: String) -> Bool:
    try:
        var f = open(path, "r")
        f.close()
        return True
    except:
        return False

# Helper to list files in a dir (via ls) and return newline-separated String
fn list_files(dir_path: String) -> String:
    try:
        var cmd = String("ls ") + dir_path
        var out = run(cmd)
        return out
    except:
        return ""

fn main() raises:
    var args = sys.argv()
    if len(args) < 2:
        print("Usage: ksdev [--wash | --load]")
        return
    
    var command = String(args[1])
    var cwd = run("pwd").strip()
    var project_root = String(cwd)
    var wash_dir = project_root + "/.wash"
    var load_dir = project_root + "/.load"
    var examples_dir = project_root + "/examples"

    if command == "--wash":
        var msg = get_icon_wash() + " Washing plugins..."
        print(msg)
        
        # Ensure .load exists
        _ = run(String("mkdir -p ") + load_dir)
        
        # We will wash contents of 'examples/' as that seems to be the source of truth
        # User mentioned battery/separator, which are in examples/
        var files_out = list_files(examples_dir)
        var lines = files_out.splitlines()
        
        for line in lines:
            var filename = line.strip()
            if filename.endswith(".rs"):
                var src_path = examples_dir + "/" + filename
                var building_msg = get_color_blue() + "Building: " + get_color_reset() + filename
                print(building_msg)
                
                var cmd = String("mojo tools/wash.mojo ") + src_path
                # We need to capture invalid output, but run() returns stdout.
                # If it fails, we might see it in stdout if we print it in wash.mojo
                var out = run(cmd)
                print(out)
                
                # Check for artifact in project root (where wash.mojo leaves it)
                var dish_name = filename.replace(".rs", ".dish")
                var dish_path = project_root + "/" + dish_name
                
                if file_exists(dish_path):
                    # Move to .load
                    var dest_path = load_dir + "/" + dish_name
                    var mv_cmd = String("mv ") + dish_path + " " + dest_path
                    _ = run(mv_cmd)
                    
                    var moved_msg = String("  -> Moved to .load/") + dish_name
                    print(moved_msg)
                else:
                    # Maybe it's already in .load if I changed wash.mojo logic (I didn't yet)
                    # output "✨ Dish ready: datetime.dish" suggests it is in CWD.
                    var warn = get_icon_warn() + " Artifact not found in root: " + dish_name
                    print(warn)

    elif command == "--load":
        var msg = get_icon_load() + " Loading plugins from .load..."
        print(msg)
        
        if not dir_exists(load_dir):
            var err = get_icon_error() + " .load directory not found"
            print(err)
            return

        var files_out = list_files(load_dir)
        var lines = files_out.splitlines()
        
        for line in lines:
            var filename = line.strip()
            if filename.endswith(".dish"):
                var artifact_path = load_dir + "/" + filename
                var loading_msg = get_color_blue() + "Loading: " + get_color_reset() + filename
                print(loading_msg)
                
                var cmd = String("ks-bin load ") + artifact_path
                # ks-bin load might need to be run? Or just copy?
                # User previously used `ks-bin load`.
                # Assuming `ks-bin` is in path or we use `cargo run`. 
                # Better to use installed binary if available, or just copy to destination?
                # "Load" command usually installs it.
                var out = run(cmd)
                print(out)

    else:
        print("Unknown command")
