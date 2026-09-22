"""Run implemented PPU library checks through the maintained test entry point."""
from pathlib import Path
import runpy
runpy.run_path(str(Path(__file__).with_name("test-ppu-library.py")),run_name="__main__")
