from python import Python, PythonObject
from sys import argv

fn main() raises:
    var os = Python.import_module("os")
    var subprocess = Python.import_module("subprocess")
    var shutil = Python.import_module("shutil")
    var builtins = Python.import_module("builtins")

    var args = argv()
    # print("DEBUG ARGS:", args[1]) 
    if len(args) < 2:
        print("Usage: ksdev [--wash | --load]")
        return

    var command = args[1]
    var project_root = os.getcwd()
    var wash_dir = os.path.join(project_root, ".wash")
    var load_dir = os.path.join(project_root, ".load")

    if command == "--wash":
        print("🧼 Washing plugins from .wash...")
        
        if not os.path.exists(wash_dir):
            print("Error: .wash directory not found.")
            return

        var wash_files = os.listdir(wash_dir)
        for i in range(len(wash_files)):
            var filename = wash_files[i]
            
            if filename.endswith(".rs"):
                var src_path = os.path.join(wash_dir, filename)
                print("Building:", filename)
                
                # Construct python list for subprocess
                var run_cmd = builtins.list()
                var _ = run_cmd.append("mojo")
                var _ = run_cmd.append("tools/wash.mojo")
                var _ = run_cmd.append(src_path)
                
                var result = subprocess.run(run_cmd, capture_output=True, text=True)
                
                if result.returncode != 0:
                    print("Error building", filename, ":")
                    print(result.stderr)
                else:
                    print("Build successful.")
                    var dish_name = filename.replace(".rs", ".dish")
                    var dish_path = os.path.join(project_root, dish_name)
                    
                    if os.path.exists(dish_path):
                        var dest_path = os.path.join(load_dir, dish_name)
                        print("Moving artifact to", dest_path)
                        var _ = shutil.move(dish_path, dest_path)
                    else:
                        print("Warning: Artifact", dish_name, "not found after build.")

    elif command == "--load":
        print("🍽️ Loading plugins from .load...")
        
        if not os.path.exists(load_dir):
            print("Error: .load directory not found.")
            return

        var load_files = os.listdir(load_dir)
        for i in range(len(load_files)):
            var filename = load_files[i]
            
            if filename.endswith(".dish"):
                var artifact_path = os.path.join(load_dir, filename)
                print("Loading:", filename)
                
                var run_cmd = builtins.list()
                var _ = run_cmd.append("ks-bin")
                var _ = run_cmd.append("load")
                var _ = run_cmd.append(artifact_path)
                
                var result = subprocess.run(run_cmd, capture_output=True, text=True)
                
                if result.returncode != 0:
                    print("Error loading", filename, ":")
                    print(result.stderr)
                else:
                    print(result.stdout)
                    
    else:
        print("Unknown command:", command)
        print("Usage: ksdev [--wash | --load]")
