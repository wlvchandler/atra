#!/usr/bin/env python3
import os
import sys
from pathlib import Path

def get_repo_root():
    script_path = Path(__file__).resolve()
    repo_dir = script_path.parent
    return repo_dir

def main():
    from atra_cli_core.utils.venv_manager import VenvManager
    
    repo_root = get_repo_root()
    args = sys.argv[1:]

    if args and args[0] == 'clean':
        from atra_cli_core.utils.cleanup import CleanupManager
        CleanupManager.clean_all()
        return
    
    if args and args[0] == 'init':
        from atra_cli_core.setup_internals import environment
        environment.setup_environment()
        print("Environment setup complete. You can now run `atra` commands directly.")
        return

    from atra_cli_core.setup_internals.environment import is_editable_install

    if not (is_editable_install() or VenvManager.is_venv_activated()):
        if VenvManager.venv_exists():
            VenvManager.exec_in_venv(args)
        else:
            print("Virtual environment not found. Please run 'atra init' first.")
            sys.exit(1)
    else:
        from atra_cli_core.utils import commands
        commands.run_cli(args)

if __name__ == '__main__':
    main()
