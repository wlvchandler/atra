import os
import shutil
from pathlib import Path
from atra_cli_core.utils.venv_manager import VenvManager

class CleanupManager:
    
    @staticmethod
    def _remove_path(path, description, relative_to=None):
        if not path.exists():
            return False
            
        rel_path = path.relative_to(relative_to) if relative_to else path
        print(f"Removing {description}: {rel_path}")
        
        if path.is_dir():
            shutil.rmtree(path)
        else:
            path.unlink()
        return True
    
    @staticmethod
    def clean_virtual_environment():
        cli_dir = VenvManager.get_cli_dir()
        result = CleanupManager._remove_path(cli_dir / 'venv', "virtual environment")
        if not result:
            print("No virtual environment found")
        return result

    @staticmethod
    def clean_generated_files():
        gen_dir = VenvManager.get_cli_dir() / 'src' / 'atra_cli_core'
        result = CleanupManager._remove_path(gen_dir / 'generated', "generated files")
        if not result:
            print("No generated files found")
        return result

    @staticmethod
    def clean_python_cache():
        cli_dir = VenvManager.get_cli_dir()
        cleaned = False
        
        for cache_dir in cli_dir.rglob('__pycache__'):
            if CleanupManager._remove_path(cache_dir, "Python cache directory", cli_dir):
                cleaned = True
        
        for pyc_file in cli_dir.rglob('*.pyc'):
            if CleanupManager._remove_path(pyc_file, "compiled Python file", cli_dir):
                cleaned = True
        
        pytest_cache = cli_dir / '.pytest_cache'
        if CleanupManager._remove_path(pytest_cache, "pytest cache", cli_dir):
            cleaned = True
                
        if not cleaned:
            print("No Python cache files found")
        return cleaned

    @staticmethod
    def clean_log_files():
        cli_dir = VenvManager.get_cli_dir()
        cleaned = False
        
        for log_file in cli_dir.rglob('*.log'):
            if CleanupManager._remove_path(log_file, "log file", cli_dir):
                cleaned = True
                
        if not cleaned:
            print("No log files found")
        return cleaned

    @staticmethod
    def clean_all():
        print("Cleaning all artifacts and cache files...")
        
        cleanup_operations = [
            CleanupManager.clean_virtual_environment,
            CleanupManager.clean_generated_files,
            CleanupManager.clean_python_cache,
            CleanupManager.clean_log_files
        ]
        
        cleaned = any(operation() for operation in cleanup_operations)
        print("Cleanup complete!" if cleaned else "Nothing to clean")
        return True
