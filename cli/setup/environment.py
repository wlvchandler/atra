import os
import sys
import subprocess
import logging
from pathlib import Path

from cli.utils.venv_manager import VenvManager
from cli.platform import configure_venv, setup_executable, print_next_steps

cli_dir = Path(__file__).resolve().parent.parent
log_file = cli_dir / 'setup.log'

logging.basicConfig(
    level=logging.INFO,
    format='%(message)s',
    handlers=[
        logging.FileHandler(log_file),
        logging.StreamHandler(sys.stdout)
    ]
)

class ColoredOutput:
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'

    @staticmethod
    def success(msg):
        return f"{ColoredOutput.GREEN}{msg}{ColoredOutput.ENDC}"

    @staticmethod
    def info(msg):
        return f"{ColoredOutput.BLUE}{msg}{ColoredOutput.ENDC}"

    @staticmethod
    def error(msg):
        return f"{ColoredOutput.RED}{msg}{ColoredOutput.ENDC}"

    @staticmethod
    def bold(msg):
        return f"{ColoredOutput.BOLD}{msg}{ColoredOutput.ENDC}"

def run_command(cmd, check=True, env=None, capture_output=True):
    cmd_str = " ".join(str(x) for x in cmd)
    logging.debug(f"Running command: {cmd_str}")

    try:
        result = subprocess.run(
            cmd,
            check=check,
            env=env,
            capture_output=capture_output,
            text=True
        )
        if result.stdout:
            logging.debug(result.stdout)
        return True
    except subprocess.CalledProcessError as e:
        logging.error(f"Command failed: {e}")
        if e.output:
            logging.debug(e.output)
        return False

def create_virtual_environment():
    cli_dir = VenvManager.get_cli_dir()
    venv_dir = VenvManager.get_venv_dir()
    
    print(ColoredOutput.info("📦 Creating virtual environment..."))
    
    try:
        subprocess.run(
            [sys.executable, "-m", "venv", str(venv_dir)],
            check=True,
            capture_output=True
        )
        print(ColoredOutput.success("✓ Virtual environment created"))
        return True
    except subprocess.CalledProcessError as e:
        print(ColoredOutput.error(f"Failed to create virtual environment: {e}"))
        return False

def configure_venv_with_help():
    venv_dir = VenvManager.get_venv_dir()
    venv_paths = VenvManager.get_venv_paths()
    
    help_message = '\n--------\n-atraCLI-\nHelp:\n\tatra -h {{book|place} -h}\n--------\n'
    
    try:
        configure_venv(venv_dir, venv_paths["activate_path"], help_message)
        return True
    except Exception as e:
        print(ColoredOutput.error(f"Failed to configure venv: {e}"))
        return False

def generate_proto_files():
    cli_dir = VenvManager.get_cli_dir()
    repo_root = VenvManager.find_repo_root(cli_dir)
    venv_paths = VenvManager.get_venv_paths()
    python_path = venv_paths["python"]
    
    if not repo_root:
        print(ColoredOutput.error("Repository root not found. Missing .atraroot marker file."))
        return False
    
    print(ColoredOutput.info("\n🔨 Generating proto files..."))
    
    generated_dir = cli_dir / "generated"
    generated_dir.mkdir(exist_ok=True)
    
    proto_script = repo_root / "scripts" / "generate_proto.py"
    
    if not proto_script.exists():
        proto_script = cli_dir / "scripts" / "generate_proto.py"
        if not proto_script.exists():
            print(ColoredOutput.error(f"Proto generator script not found"))
            # continue anyway
            return True
    
    result = run_command([str(python_path), str(proto_script)])
    
    if result:
        print(ColoredOutput.success("✓ Proto files generated"))
        return True
    else:
        print(ColoredOutput.error("Failed to generate proto files"))
        return False

def ensure_script_executable():
    cli_dir = VenvManager.get_cli_dir()
    repo_root = VenvManager.find_repo_root(cli_dir)
    
    if not repo_root:
        print(ColoredOutput.error("Repository root not found. Missing .atraroot marker file."))
        return False
    
    if setup_executable(cli_dir, repo_root):
        print(ColoredOutput.success("✓ Made atra script executable"))
        return True
    else:
        print(ColoredOutput.error("Failed to make script executable"))
        return False

def setup_environment():
    cli_dir = VenvManager.get_cli_dir()
    repo_root = VenvManager.find_repo_root(cli_dir)
    
    print(ColoredOutput.bold("\n🔧 Setting up development environment...\n"))
    print(ColoredOutput.info(f"CLI directory: {cli_dir}"))
    if repo_root:
        print(ColoredOutput.info(f"Repository root: {repo_root}"))
    
    if not create_virtual_environment():
        return False
    
    if not configure_venv_with_help():
        return False
    
    print(ColoredOutput.info("\n📥 Installing requirements..."))
    if not VenvManager.install_requirements():
        print(ColoredOutput.error("Failed to install requirements"))
        return False
    print(ColoredOutput.success("✓ Dependencies installed"))
    
    if not generate_proto_files():
        return False
    
    if not ensure_script_executable():
        return False
    
    venv_paths = VenvManager.get_venv_paths()
    print_next_steps(venv_paths["activate_cmd"], repo_root or cli_dir)
    
    print(ColoredOutput.bold("\n✨ Setup complete! You can now use the atra command."))
    print(ColoredOutput.info("\nExample commands:"))
    print("   atra book        # Show the orderbook")
    print("   atra trades      # Show recent trades")
    print("   atra buy 10@100  # Place a buy order: 10 units at price 100")
    print("\n")
    
    return True
