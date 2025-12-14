from python import Python
from python import PythonObject

fn get_color(code: String) -> String:
    return "\033[" + code + "m"

fn is_ignored_name(name: String) -> Bool:
    if name == ".git": return True
    if name == "target": return True
    if name == ".venv": return True
    if name == "node_modules": return True
    if name == ".wash": return True
    if name == ".load": return True
    if name == ".serena": return True
    if name == "in-memoria-vectors.db": return True
    if name == "__pycache__": return True
    if name == ".vscode": return True
    if name == ".idea": return True
    return False

fn is_path_ignored(path: String) -> Bool:
    var parts = path.split("/")
    for i in range(len(parts)):
        # Convert slice to String
        if is_ignored_name(String(parts[i])):
            return True
    return False

fn main() raises:
    var sys = Python.import_module("sys")
    var os = Python.import_module("os")
    var builtins = Python.import_module("builtins")
    
    # --- ANSI Colors ---
    var RESET = get_color("0")
    var BOLD = get_color("1")
    var GRAY = get_color("90")
    var CYAN = get_color("36")
    var GREEN = get_color("32")
    var ORANGE = "\033[38;5;208m"
    var RED = "\033[31m"

    var py_none = Python.evaluate("None")

    # Extension Mapping
    var lang_config = Python.dict()
    lang_config['.rs']   = Python.tuple('Rust',   '//', '/*', '*/')
    lang_config['.mojo'] = Python.tuple('Mojo',   '#',  py_none, py_none)
    lang_config['.py']   = Python.tuple('Python', '#',  '"""', '"""')
    lang_config['.toml'] = Python.tuple('Config', '#',  py_none, py_none)
    lang_config['.md']   = Python.tuple('Text',   py_none, '<!--', '-->')
    lang_config['.sh']   = Python.tuple('Shell',  '#',  py_none, py_none)
    lang_config['.yml']  = Python.tuple('Config', '#',  py_none, py_none)
    lang_config['.yaml'] = Python.tuple('Config', '#',  py_none, py_none)
    lang_config['.json'] = Python.tuple('Config', py_none, py_none, py_none)

    # CLI Args
    var argv = sys.argv
    var root_dir = String(".")
    if len(argv) > 1:
        root_dir = String(argv[1])

    var file_stats = Python.list() 
    var lang_stats = Python.dict()

    print(BOLD + "📊 Project Code Statistics (LOC)" + RESET)
    print(GRAY + "Excluding comments, whitespace, and ignored dirs" + RESET + "\n")

    # Walk directory
    for root_tuple in os.walk(root_dir):
        var root = String(root_tuple[0])
        var dirs = root_tuple[1]
        var files = root_tuple[2]
        
        # 1. Skip if current root is inside an ignored path
        if is_path_ignored(root):
            var length = Int(builtins.len(dirs))
            while length > 0:
                _ = dirs.pop()
                length -= 1
            continue

        # 2. Filter dirs in-place to prevent descending
        var i = Int(builtins.len(dirs)) - 1
        while i >= 0:
            var d = dirs[i]
            # Use String(builtins.str(d)) to ensure conversion from PythonObject
            if is_ignored_name(String(builtins.str(d))):
                _ = dirs.pop(i)
            i -= 1
            
        for filename in files:
            var file_obj = filename
            var split = os.path.splitext(file_obj)
            var ext = split[1]
            
            if ext in lang_config:
                var path = os.path.join(root, file_obj)
                
                # Double check path ignore
                if is_path_ignored(String(path)):
                    continue

                var config = lang_config[ext]
                var lang_name = config[0]
                var line_comment = config[1]
                var block_start = config[2]
                var block_end = config[3]
                
                var code_lines = 0
                var in_block = False
                
                try:
                    var f = builtins.open(path, "r", encoding="utf-8", errors="ignore")
                    var content = f.read()
                    f.close()
                    
                    var lines = content.splitlines()
                    for line_obj in lines:
                        var line = String(line_obj).strip()
                        if len(line) == 0:
                            continue
                            
                        # Block handling
                        if block_start is not py_none and block_end is not py_none:
                            var bs = String(block_start)
                            var be = String(block_end)
                            
                            if in_block:
                                if be in line:
                                    in_block = False
                                continue
                            
                            if bs in line:
                                if be in line and line.find(be) > line.find(bs):
                                    pass 
                                else:
                                    in_block = True
                                continue
                        
                        # Line comment
                        if line_comment is not py_none:
                            var lc = String(line_comment)
                            if line.startswith(lc):
                                continue
                                
                        code_lines += 1
                        
                except:
                    code_lines = 0
                
                if code_lines > 0:
                    var stat_entry = Python.tuple(code_lines, path, lang_name)
                    file_stats.append(stat_entry)
                    
                    var current = 0
                    if lang_name in lang_stats:
                        current = Int(lang_stats[lang_name])
                    lang_stats[lang_name] = current + code_lines

    # --- Summary ---
    var total_loc = 0
    var lang_keys = builtins.list(lang_stats)
    for k in lang_keys:
        total_loc += Int(lang_stats[k])
        
    var key_func = Python.evaluate("lambda x: x[1]")
    var sorted_langs = builtins.sorted(lang_stats.items(), key=key_func, reverse=True)
    
    # Header
    var h_lang = String(builtins.str("Language").ljust(12))
    var h_loc = String(builtins.str("LOC").rjust(10))
    print(BOLD + h_lang + " " + h_loc + "   " + "%" + RESET)
    print(GRAY + "-"*30 + RESET)
    
    for item in sorted_langs:
        var py_lang = item[0]
        var py_loc = item[1]
        var loc_int = Int(py_loc)
        var percent = 0.0
        if total_loc > 0:
            percent = (loc_int / total_loc) * 100
            
        var color = CYAN
        var lang_str = String(py_lang)
        if lang_str == 'Rust': color = ORANGE
        elif lang_str == 'Mojo': color = RED
        elif lang_str == 'Python': color = GREEN
        
        var fmt_lang = String(builtins.str(py_lang).ljust(12))
        var fmt_loc = String(builtins.str(py_loc).rjust(10))
        
        var p_int = Int(percent * 10)
        var p_dec = p_int % 10
        var p_whole = p_int // 10
        var p_py_str = builtins.str(p_whole) + "." + builtins.str(p_dec)
        var fmt_pct = String(p_py_str.rjust(5))
        
        print(color + fmt_lang + " " + fmt_loc + RESET + "   " + GRAY + fmt_pct + "%" + RESET)

    print(GRAY + "-"*30 + RESET)
    var fmt_total = String(builtins.str(total_loc).rjust(10))
    var fmt_label = String(builtins.str("TOTAL").ljust(12))
    print(BOLD + fmt_label + " " + fmt_total + RESET + "\n")

    # --- Top 20 Files ---
    print(BOLD + "🏆 Top 20 Largest Files" + RESET)
    print(GRAY + "-"*60 + RESET)
    
    var sort_key = Python.evaluate("lambda x: x[0]")
    file_stats.sort(key=sort_key, reverse=True)
    
    var limit = 20
    if len(file_stats) < 20:
        limit = len(file_stats)
        
    for i in range(limit):
        var entry = file_stats[i]
        var py_loc = entry[0]
        var py_path = entry[1]
        var py_lang = entry[2]
        
        var rel_path_py = os.path.relpath(py_path, root_dir)
        var lang_str = String(py_lang)
        var color = CYAN
        if lang_str == 'Rust': color = ORANGE
        elif lang_str == 'Mojo': color = RED
        
        var idx_str = String(i + 1) + "."
        var fmt_idx = String(builtins.str(idx_str).ljust(4))
        var fmt_loc = String(builtins.str(py_loc).rjust(6))
        
        print(fmt_idx + color + fmt_loc + RESET + "  " + String(rel_path_py))