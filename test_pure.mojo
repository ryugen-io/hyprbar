from pathlib import Path
from sys import argv
from subprocess import run

fn main() raises:
    print("Testing generic imports")
    var p = Path("test_pure.mojo")
    print("Path exists:", p.exists())
    
    # Check subprocess
    print("Running ls:")
    var out = run("ls -F")
    print(out)
