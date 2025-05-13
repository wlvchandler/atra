import os
from pathlib import Path

def get_venv_paths(venv_dir):
    return {
        "python": venv_dir / "bin" / "python",
        "pip": venv_dir / "bin" / "pip",
        "activate_path": venv_dir / "bin" / "activate",
        "activate_cmd": f"source {venv_dir}/bin/activate",
    }

def configure_venv(venv_dir, activate_file, help_message):
    with open(activate_file, "a") as f:
        f.write(f'\nprintf "\\033[1;32m{help_message}\\033[0m\\n"\n')

def setup_executable(cli_dir, repo_root):
    atra_path = cli_dir / "atra"
    if atra_path.exists():
        atra_path.chmod(atra_path.stat().st_mode | 0o111)

    bin_atra = repo_root / "bin" / "atra"
    if not bin_atra.exists() or not bin_atra.is_symlink():
        print(f"Note: bin/atra symlink not found or not a symlink in repository")
    return True

def print_next_steps(activate_cmd, repo_root):
    bin_path = repo_root / "bin"
    rel_bin_path = Path(os.path.relpath(bin_path))

    print("\n✨ Setup complete! Next steps:")
    print(f"\n1. Activate virtual environment:")
    print(f"   {activate_cmd}")
    print(f"\n2. Run the CLI:")
    print(f"   {rel_bin_path}/atra <command>")
    print("\n")
