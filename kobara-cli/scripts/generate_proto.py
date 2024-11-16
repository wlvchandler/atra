#!/usr/bin/env python3
import subprocess
import shutil
from pathlib import Path

def main():
    project_dir = Path(__file__).parent.parent
    proto_file = project_dir.parent / "kobara-proto" / "proto" / "orderbook.proto"
    generated_dir = project_dir / "generated"

    if generated_dir.exists():
        shutil.rmtree(generated_dir)
    generated_dir.mkdir()

    subprocess.run([
        "python", "-m", "grpc_tools.protoc",
        f"-I{proto_file.parent}",
        f"--python_out={generated_dir}",
        f"--grpc_python_out={generated_dir}",
        str(proto_file)
    ], check=True)

    with open(generated_dir / "__init__.py", "w") as f:
        f.write("from . import orderbook_pb2, orderbook_pb2_grpc\n")

    # patch some import issues in generated files
    grpc_file = generated_dir / "orderbook_pb2_grpc.py"
    content = grpc_file.read_text()
    fixed_content = content.replace(
        "import orderbook_pb2 as orderbook__pb2",
        "from . import orderbook_pb2 as orderbook__pb2"
    )
    grpc_file.write_text(fixed_content)

if __name__ == "__main__":
    main()
