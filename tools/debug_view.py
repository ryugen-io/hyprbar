#!/usr/bin/env python3
"""
kitchnsink Debug View Tool
Author: Ryu
Description: Visual debug helper and config extractor.
Dependencies: typer, rich, tomli, mss (or grim for Wayland)
"""

import os
import glob
import json
import subprocess
import sys
from pathlib import Path
from datetime import datetime
from typing import Optional

try:
    import typer
    from rich.console import Console
    from rich.syntax import Syntax
    from rich.panel import Panel
    import tomli
    from PIL import Image
except ImportError:
    print("❌ Missing dependencies. Run: pip install typer rich tomli types-toml pillow")
    sys.exit(1)

app = typer.Typer()
console = Console()

CONFIG_DIR = Path.home() / ".config" / "kitchnsink"
SINK_TOML = CONFIG_DIR / "sink.toml"
# Screenshots in .screenshots directory relative to this script
SCREENSHOT_DIR = Path(__file__).parent / ".screenshots"

def merge_configs(base_config: dict) -> dict:
    """Recursively merge configs based on 'include' glob patterns."""
    includes = base_config.get("include", [])
    if not includes:
        return base_config

    merged = base_config.copy()
    
    for pattern in includes:
        # Expand glob relative to config dir
        full_pattern = CONFIG_DIR / pattern
        for file_path in glob.glob(str(full_pattern)):
            try:
                with open(file_path, "rb") as f:
                    dish_config = tomli.load(f)
                    
                    # Merge logic (simplistic shallow merge for 'dish' key)
                    if "dish" in dish_config:
                        if "dish" not in merged:
                            merged["dish"] = {}
                        merged["dish"].update(dish_config["dish"])
                        
                    # Merge layout if present
                    if "layout" in dish_config:
                        if "layout" not in merged:
                            merged["layout"] = {}
                        merged["layout"].update(dish_config["layout"])

            except Exception as e:
                console.print(f"[red]Failed to load included config {file_path}: {e}[/red]")

    return merged

@app.command()
def inspect(
    screenshot: bool = typer.Option(True, help="Take a screenshot of the bar area"),
    dump_json: bool = typer.Option(False, help="Dump merged config as JSON"),
):
    """Inspect current kitchnsink state (Config + Visual)."""
    console.rule("[bold blue]kitchnsink Debug View[/bold blue]")

    # 1. Config Extraction
    if not SINK_TOML.exists():
        console.print(f"[red]Config not found at {SINK_TOML}[/red]")
        return

    try:
        with open(SINK_TOML, "rb") as f:
            raw_config = tomli.load(f)
        
        final_config = merge_configs(raw_config)
        
        if dump_json:
            print(json.dumps(final_config, indent=2))
            # Don't return, allow screenshot to proceed if enabled

        # Display Config Summary
        console.print(Panel(
            f"Config Path: {SINK_TOML}\n"
            f"Dishes: {', '.join(final_config.get('dish', {}).keys())}\n"
            f"Layout: {final_config.get('layout', {}).get('width', 'Unknown')}",
            title="Configuration",
            border_style="green"
        ))
        
        # Show 'include' patterns
        if "include" in raw_config:
            console.print(f"[yellow]Includes:[/yellow] {raw_config['include']}")

    except Exception as e:
        console.print(f"[bold red]Error parsing config:[/bold red] {e}")

    # 2. Screenshot (Smart Crop)
    if screenshot:
        SCREENSHOT_DIR.mkdir(parents=True, exist_ok=True)
        # Clean up old screenshots
        for file in SCREENSHOT_DIR.glob("*"):
            if file.is_file():
                file.unlink()
        console.print(f"[dim]Cleaned up old screenshots in {SCREENSHOT_DIR}[/dim]")
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = SCREENSHOT_DIR / f"debug_{timestamp}.png"
        
        console.print("\n[bold cyan]📸 Capturing Bar...[/bold cyan]")
        
        # 1. Get Bar Height from Config
        layout_cfg = final_config.get("layout", {})
        raw_height = layout_cfg.get("height", 30)
        
        # Handle string inputs like "30px" or integers
        try:
            if isinstance(raw_height, str):
                bar_height = int(raw_height.lower().replace("px", "").strip())
            else:
                bar_height = int(raw_height)
        except ValueError:
            bar_height = 30 # Fallback
            
        console.print(f"[dim]Expecting bar height: {bar_height}px[/dim]")

        # 2. Capture & Crop
        try:
             # Capture full screen to temp
             temp_shot = SCREENSHOT_DIR / "_temp_full.png"
             subprocess.run(["grim", str(temp_shot)], check=True)
             
             # Crop using Pillow
             with Image.open(temp_shot) as img:
                 width, height = img.size
                 # Crop bottom N pixels
                 # Box: (left, upper, right, lower)
                 # Bar is at bottom: upper = total_h - bar_h
                 crop_box = (0, height - bar_height, width, height)
                 
                 bar_img = img.crop(crop_box)
                 bar_img.save(filename)
                 
             # Cleanup
             temp_shot.unlink(missing_ok=True)
             
             console.print(f"[green]Saved exact bar screenshot to:[/green] {filename}")
             
        except FileNotFoundError:
             console.print("[red]❌ 'grim' not found. Please install grim for screenshots on Wayland.[/red]")
        except Exception as e:
             console.print(f"[red]Screenshot failed: {e}[/red]")

if __name__ == "__main__":
    app()
