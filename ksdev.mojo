from python import Python, PythonObject
from sys import argv

fn main() raises:
    var os = Python.import_module("os")
    var subprocess = Python.import_module("subprocess")
    var shutil = Python.import_module("shutil")
    var builtins = Python.import_module("builtins")

    var args = argv()
    from pathlib import Path
    # print("DEBUG ARGS:", args[1]) 
    if len(args) < 2:
        print("Usage: ksdev [--wash | --load]")
        return

    var command = args[1]
    var project_root = os.getcwd()
    var wash_dir = os.path.join(project_root, ".wash")
    var load_dir = os.path.join(project_root, ".load")

    # Nerd Font Icons & Colors
    var C_RESET = "\033[0m"
    var C_RED = "\033[31m"
    var C_GREEN = "\033[32m"
    var C_YELLOW = "\033[33m"
    var C_BLUE = "\033[34m"
    var C_MAGENTA = "\033[35m"
    var C_CYAN = "\033[36m"

    var ICON_SEARCH = C_BLUE + "" + C_RESET
    var ICON_CHECK = C_GREEN + "" + C_RESET
    var ICON_ERROR = C_RED + "" + C_RESET
    var ICON_WARN = C_YELLOW + "" + C_RESET
    var ICON_WASH = C_CYAN + "" + C_RESET
    var ICON_LOAD = C_MAGENTA + "" + C_RESET

    if command == "--wash":
        print(ICON_WASH + " Washing plugins from .wash...")
        
        if not os.path.exists(wash_dir):
            print(ICON_ERROR + " Error: .wash directory not found.")
            return

        var wash_files = os.listdir(wash_dir)
        for i in range(len(wash_files)):
            var filename = wash_files[i]
            
            if filename.endswith(".rs"):
                var src_path = os.path.join(wash_dir, filename)
                print(C_BLUE + "Building:" + C_RESET, filename)
                
                var run_cmd = builtins.list()
                var _ = run_cmd.append("mojo")
                var _ = run_cmd.append("tools/wash.mojo")
                var _ = run_cmd.append(src_path)
                
                var result = subprocess.run(run_cmd, capture_output=True, text=True)
                
                if result.returncode != 0:
                    print(ICON_ERROR + " Error building", filename, ":")
                    print(result.stderr)
                else:
                    print(ICON_CHECK + " Build successful.")
                    var dish_name = filename.replace(".rs", ".dish")
                    var dish_path = os.path.join(project_root, dish_name)
                    
                    if os.path.exists(dish_path):
                        var dest_path = os.path.join(load_dir, dish_name)
                        print("Moving artifact to", dest_path)
                        var _ = shutil.move(dish_path, dest_path)
                    else:
                        print(ICON_WARN + " Warning: Artifact", dish_name, "not found after build.")

    elif command == "--load":
        print(ICON_LOAD + " Loading plugins from .load...")
        
        if not os.path.exists(load_dir):
            print(ICON_ERROR + " Error: .load directory not found.")
            return

        var load_files = os.listdir(load_dir)
        for i in range(len(load_files)):
            var filename = load_files[i]
            
            if filename.endswith(".dish"):
                var artifact_path = os.path.join(load_dir, filename)
                print(C_BLUE + "Loading:" + C_RESET, filename)
                
                var run_cmd = builtins.list()
                var _ = run_cmd.append("ks-bin")
                var _ = run_cmd.append("load")
                var _ = run_cmd.append(artifact_path)
                
                var result = subprocess.run(run_cmd, capture_output=True, text=True)
                
                if result.returncode != 0:
                    print(ICON_ERROR + " Error loading", filename, ":")
                    print(result.stderr)
                else:
                    print(result.stdout)
                    
    elif command == "--check-dish":
        if len(args) < 3:
            print("Usage: ksdev --check-dish <file_path>")
            return

        var file_path_str = args[2]
        var file_path = Path(file_path_str)
        
        if not file_path.exists():
            print(ICON_ERROR + " Error: File not found:", file_path_str)
            return

        print(ICON_SEARCH + " Checking metadata for:", C_CYAN + file_path_str + C_RESET)
        
        # Pure Mojo file reading
        var content: String
        # Using try/except block for safety although read_text might raise
        try:
            content = file_path.read_text()
        except e:
            print(ICON_ERROR + " Failed to read file:", e)
            return
            
        var lines = content.split("\n")
        
        var found_name = False
        var found_version = False
        var found_author = False
        var found_desc = False

        for i in range(len(lines)):
            var line = lines[i].strip()
            
            if line.startswith("//! Name:"):
                found_name = True
            elif line.startswith("//! Version:"):
                found_version = True
            elif line.startswith("//! Author:"):
                found_author = True
            elif line.startswith("//! Description:"):
                found_desc = True
        
        var all_ok = True
        
        if not found_name:
            print(ICON_ERROR + " Missing '//! Name: ...'")
            all_ok = False
        if not found_version:
            print(ICON_ERROR + " Missing '//! Version: ...'")
            all_ok = False
        if not found_author:
            print(ICON_ERROR + " Missing '//! Author: ...'")
            all_ok = False
        if not found_desc:
            print(ICON_ERROR + " Missing '//! Description: ...'")
            all_ok = False

        if all_ok:
            print(ICON_CHECK + " Metadata valid!")
        else:
            print(ICON_WARN + " Please add the missing metadata fields to your dish.")

    else:
        print(ICON_ERROR + " Unknown command:", command)
        print("Usage: ksdev [--wash | --load | --check-dish <file>]")
