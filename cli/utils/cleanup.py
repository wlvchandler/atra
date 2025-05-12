import os
import shutil
from pathlib import Path

class CleanupManager:
    
    @staticmethod
    def get_cli_dir():
        return Path(__file__).resolve().parent.parent
    
    @staticmethod
    def clean_virtual_environment():
        cli_dir = CleanupManager.get_cli_dir()
        venv_dir = cli_dir / 'venv'        
        if venv_dir.exists():
            print(f"Removing virtual environment...")
            shutil.rmtree(venv_dir)
            return True
        else:
            print("No virtual environment found")
            return False
    
    @staticmethod
    def clean_generated_files():
        cli_dir = CleanupManager.get_cli_dir()
        generated_dir = cli_dir / 'generated'
        if generated_dir.exists():
            print(f"Removing generated files...")
            shutil.rmtree(generated_dir)
            return True
        else:
            print("No generated files found")
            return False
    
    @staticmethod
    def clean_python_cache():
        cli_dir = CleanupManager.get_cli_dir()
        cleaned = False        
        for pycache_dir in cli_dir.glob('**/__pycache__'):
            if pycache_dir.is_dir():
                print(f"Removing {pycache_dir.relative_to(cli_dir)}...")
                shutil.rmtree(pycache_dir)
                cleaned = True        
        for pyc_file in cli_dir.glob('**/*.pyc'):
            if pyc_file.is_file():
                print(f"Removing {pyc_file.relative_to(cli_dir)}...")
                pyc_file.unlink()
                cleaned = True        
        pytest_cache = cli_dir / '.pytest_cache'
        if pytest_cache.exists():
            print("Removing pytest cache...")
            shutil.rmtree(pytest_cache)
            cleaned = True
        if not cleaned:
            print("No Python cache files found")        
        return cleaned
    
    @staticmethod
    def clean_log_files():
        cli_dir = CleanupManager.get_cli_dir()
        cleaned = False
        for log_file in cli_dir.glob('**/*.log'):
            if log_file.is_file():
                print(f"Removing {log_file.relative_to(cli_dir)}...")
                log_file.unlink()
                cleaned = True
        if not cleaned:
            print("No log files found")
        return cleaned
    
    @staticmethod
    def clean_all():
        print("Cleaning all artifacts and cache files...")
        results = [
            CleanupManager.clean_virtual_environment(),
            CleanupManager.clean_generated_files(),
            CleanupManager.clean_python_cache(),
            CleanupManager.clean_log_files()
        ]
        
        if any(results):
            print("Cleanup complete!")
        else:
            print("Nothing to clean")
        
        return True
