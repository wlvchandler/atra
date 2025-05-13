import os
import sys
import subprocess
from pathlib import Path
from atra_cli_core.atra_platform import get_venv_paths

class VenvManager:
    @staticmethod
    def find_repo_root(start_dir=None):
        if start_dir is None:
            start_dir = VenvManager.get_cli_dir().parent
        current = Path(start_dir).resolve()
        while True:
            if (current / ".atraroot").exists():
                return current
            parent_path = current.parent
            if parent_path == current:
                break
        return None
    
    @staticmethod
    def get_cli_dir():
        return Path(__file__).resolve().parent.parent.parent.parent
    
    @staticmethod
    def get_venv_dir():
        return VenvManager.get_cli_dir() / 'venv'
    
    @staticmethod
    def is_venv_activated():
        return (hasattr(sys, 'real_prefix') or 
                (hasattr(sys, 'base_prefix') and sys.base_prefix != sys.prefix))
    
    @staticmethod
    def venv_exists():
        return VenvManager.get_venv_dir().is_dir()
    
    @staticmethod
    def get_venv_paths():
        venv_dir = VenvManager.get_venv_dir()
        return get_venv_paths(venv_dir)
    
    @staticmethod
    def exec_in_venv(args: list[str]):
        venv_paths = VenvManager.get_venv_paths()
        python_exe = str(venv_paths["python"])
        cli_pkg_root = VenvManager.get_cli_dir()
        main_script = cli_pkg_root / "src" / "atra_cli_core" / "__main__.py"
        if not main_script.is_file():
            print(f"ERROR: Main script for re-execution not found at {main_script}", file=sys.stderr)
            sys.exit(1)
        cmd_list = [python_exe, str(main_script)] + args
        current_env = os.environ.copy()
        python_path_addition = str(cli_pkg_root / "src")
        if "PYTHONPATH" in current_env:
            current_env["PYTHONPATH"] = f"{python_path_addition}{os.pathsep}{current_env['PYTHONPATH']}"
        else:
            current_env["PYTHONPATH"] = python_path_addition
        try:
            os.execve(python_exe, cmd_list, current_env)
        except OSError:
            print(f"ERROR: os.execve failed when trying to run Python from venv.", file=sys.stderr)
            sys.exit(1)
    
    @staticmethod
    def install_requirements():
        cli_dir = VenvManager.get_cli_dir()
        venv_paths = VenvManager.get_venv_paths()
        requirements_file = cli_dir / "requirements.txt"
        
        if not requirements_file.exists():
            print(f"Error: {requirements_file} not found")
            return False
        
        cmd = [str(venv_paths["pip"]), "install", "-r", str(requirements_file)]
        
        try:
            subprocess.run(cmd, check=True, capture_output=True, text=True)
            return True
        except subprocess.CalledProcessError as e:
            print(f"Failed to install requirements. Error: {e}")
            print(f"Stdout:\n{e.stdout}")
            print(f"Stderr:\n{e.stderr}")
            return False
