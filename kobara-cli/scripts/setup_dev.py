#!/usr/bin/env python3
import os
import subprocess
import sys
from pathlib import Path

def run_command(cmd, check=True, env=None):
    cmd_str = " ".join(str(x) for x in cmd)
    print(f"Running: {cmd_str}")

    try:
        subprocess.run(cmd, check=check, env=env)
        return True
    except subprocess.CalledProcessError as e:
        print(f"Command failed: {e}")
        return False

def main():
    project_dir = Path(__file__).parent.parent
    venv_dir = project_dir / "venv"
    generated_dir = project_dir / "generated"

    print("Creating virtual environment...")
    subprocess.run([sys.executable, "-m", "venv", venv_dir], check=True)

    if os.name == "nt":  # Windows
        python_path = venv_dir / "Scripts" / "python.exe"
        pip_path = venv_dir / "Scripts" / "pip.exe"
    else:  # Unix
        python_path = venv_dir / "bin" / "python"
        pip_path = venv_dir / "bin" / "pip"

    requirements_file = project_dir / "requirements.txt"
    run_command([pip_path, "install", "-r", requirements_file])

    generated_dir.mkdir(exist_ok=True)

    proto_script = project_dir / "scripts" / "generate_proto.py"
    run_command([python_path, proto_script])

    print("\nSetup complete. Run:")
    if os.name == "nt":
        print(r".\venv\Scripts\activate")
    else:
        print("source venv/bin/activate")
    print("python cli.py <command>")

if __name__ == "__main__":
    main()
