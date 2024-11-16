#!/usr/bin/env python3
from pathlib import Path
import subprocess

def main():
    project_dir = Path(__file__).parent.parent
    proto_file = project_dir.parent / "kobara-proto" / "proto" / "orderbook.proto"
    generated_dir = project_dir / "generated"
    generated_dir.mkdir(exist_ok=True)

    venv_python = project_dir / "venv" / "bin" / "python"

    subprocess.run([
        str(venv_python),
        "-m",
        "grpc_tools.protoc",
        f"-I{proto_file.parent}",
        f"--python_out={generated_dir}",
        f"--grpc_python_out={generated_dir}",
        str(proto_file)
    ], check=True)

    (generated_dir / "__init__.py").touch()

if __name__ == "__main__":
    main()
