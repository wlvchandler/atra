import os
import sys
import subprocess
import logging
from pathlib import Path
import importlib.resources

from atra_cli_core.utils.venv_manager import VenvManager
from atra_cli_core.atra_platform import configure_venv, setup_executable, print_next_steps

# cli_dir = Path(__file__).resolve().parent.parent
# log_file = cli_dir / 'setup.log'

log_file_parent_dir = VenvManager.get_cli_dir()
log_file = log_file_parent_dir / 'setup.log'

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
        result = subprocess.run(cmd,check=check, env=env, capture_output=capture_output, text=True)
        if result.stdout:
            logging.debug(result.stdout)
        return True
    except subprocess.CalledProcessError as e:
        logging.error(f"Command failed: {e}")
        if e.output:
            logging.debug(e.output)
        return False

    
def create_virtual_environment():
    venv_dir = VenvManager.get_venv_dir()
    print(ColoredOutput.info("📦 Creating virtual environment..."))
    success = run_command([sys.executable, "-m", "venv", str(venv_dir)])
    print(ColoredOutput.success("✓ Virtual environment created") if success else ColoredOutput.error("Failed to create virtual environment"))
    return success
    
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
    venv_paths = VenvManager.get_venv_paths()
    python_exe = Path(venv_paths["python"])

    repo_root = VenvManager.find_repo_root()
    if not repo_root:
        print(ColoredOutput.error("Repository root not found. Missing .atraroot marker file."))
        return False

    proto_source = repo_root / "atra-proto" / "proto"
    if not proto_source.is_dir():
        print(ColoredOutput.error(f"Source proto directory not found at: {proto_source}"))
        return False

    generated_dir = Path(__file__).resolve().parent.parent / "generated"
    try:
        with importlib.resources.path('atra_cli_core.scripts', 'generate_proto.py') as script_path:
            cmd = [
                str(python_exe),
                str(script_path),
                "--proto_input_dir", str(proto_source),
                "--generated_output_dir", str(generated_dir)
            ]
            success = run_command(cmd)
            print(ColoredOutput.success("✓ Proto files generated") if success else ColoredOutput.error("Proto generation script failed"))
            return success
    except FileNotFoundError:
        print(ColoredOutput.error("The 'generate_proto.py' script was not found."))
        return False
    except Exception as e:
        print(ColoredOutput.error(f"Proto generation error: {e}"))
        return False

def ensure_script_executable():
    cli_dir = VenvManager.get_cli_dir()
    repo_root = VenvManager.find_repo_root()
    if not repo_root:
        print(ColoredOutput.error("Repository root not found. Missing .atraroot marker file."))
        return False
    success = setup_executable(cli_dir, repo_root)
    print(ColoredOutput.success("✓ Made atra script executable") if success else
          ColoredOutput.error("Failed to make script executable"))
    return success

def setup_environment():
    cli_dir = VenvManager.get_cli_dir()
    repo_root = VenvManager.find_repo_root()
    
    print(ColoredOutput.bold("\n🔧 Setting up development environment...\n"))
    print(ColoredOutput.info(f"CLI directory: {cli_dir}"))
    if repo_root:
        print(ColoredOutput.info(f"Repository root: {repo_root}"))
    
    steps = [
        (create_virtual_environment, None),
        (configure_venv_with_help, None),
        (lambda: print(ColoredOutput.info("\n📥 Installing requirements...")) or True, None), 
        (VenvManager.install_requirements, "✓ Dependencies installed"),
        (generate_proto_files, None),
        (ensure_script_executable, None)
    ]
    
    for step_func, success_msg in steps:
        result = step_func()
        if success_msg and result:
            print(ColoredOutput.success(success_msg))
        if not result:
            logging.error(f"Error in step {step_func.__name__}")
            return False
    
    venv_paths = VenvManager.get_venv_paths()
    print_next_steps(venv_paths["activate_cmd"], repo_root or cli_dir)
    
    print(ColoredOutput.bold("\n✨ Setup complete! You can now use the atra command."))
    print(ColoredOutput.info("\nExample commands:"))
    print("   atra book        # Show the orderbook")
    print("   atra trades      # Show recent trades")
    print("   atra buy 10@100  # Place a buy order: 10 units at price 100")
    
    return True
