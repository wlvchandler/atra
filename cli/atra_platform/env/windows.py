import os
import subprocess
from pathlib import Path

def get_venv_paths(venv_dir):
    return {
        "python": venv_dir / "Scripts" / "python.exe",
        "pip": venv_dir / "Scripts" / "pip.exe",
        "activate_path": venv_dir / "Scripts" / "activate.bat",
        "activate_cmd": f".\\venv\\Scripts\\activate",
    }

def configure_venv(venv_dir, activate_file, help_message):
    with open(activate_file, "a") as f:
        f.write(f"\necho {help_message}\n")

def setup_executable(cli_dir, repo_root):
    bin_atra = repo_root / "bin" / "atra"
    if not bin_atra.exists():
        print("Note: bin/atra script not found in repository")

    # check if Python Launcher is available
    try:
        subprocess.run(["py", "--version"], check=True, capture_output=True)
        print("✓ Python Launcher (py) is available for executing scripts")
    except (subprocess.CalledProcessError, FileNotFoundError):
        pass

    return True

def print_next_steps(activate_cmd, repo_root):
    bin_path = repo_root / "bin"
    rel_bin_path = Path(os.path.relpath(bin_path))

    print("\n✨ Setup complete! Next steps:")
    print(f"\n1. Activate virtual environment:")
    print(f"   {activate_cmd}")
    print(f"\n2. Run the CLI:")
    print(f"   python {rel_bin_path}\\atra <command>")
    print("   # or if Python Launcher is installed:")
    print(f"   py {rel_bin_path}\\atra <command>")
    print("\n")
