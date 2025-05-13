import os
import sys
import subprocess
from pathlib import Path
from cli.atra_platform import get_venv_paths

class VenvManager:
    @staticmethod
    def find_repo_root(start_dir=None):
        if start_dir is None:
            start_dir = Path.cwd()
        current = Path(start_dir).resolve()
        while current != current.parent:
            if (current / ".atraroot").exists():
                return current
            current = current.parent
        return None
    
    @staticmethod
    def get_cli_dir():
        return Path(__file__).resolve().parent.parent
    
    @staticmethod
    def get_venv_dir():
        cli_dir = VenvManager.get_cli_dir()
        return cli_dir / 'venv'
    
    @staticmethod
    def is_venv_activated():
        return (hasattr(sys, 'real_prefix') or 
                (hasattr(sys, 'base_prefix') and sys.base_prefix != sys.prefix))
    
    @staticmethod
    def venv_exists():
        venv_dir = VenvManager.get_venv_dir()
        return venv_dir.exists()
    
    @staticmethod
    def get_venv_paths():
        venv_dir = VenvManager.get_venv_dir()
        return get_venv_paths(venv_dir)
    
    @staticmethod
    def exec_in_venv(args):
        venv_paths = VenvManager.get_venv_paths()
        python_path = str(venv_paths["python"])
        os.execv(python_path, [python_path, "-m", "cli"] + args)
    
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
            subprocess.run(cmd, check=True)
            return True
        except subprocess.CalledProcessError:
            return False
