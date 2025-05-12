FROM python:3.10-slim

WORKDIR /usr/src/app

RUN apt-get update && \
    apt-get install -y protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

COPY atra-proto/proto /usr/src/app/atra-proto/proto

COPY cli /usr/src/app/cli

WORKDIR /usr/src/app/cli
RUN pip install --no-cache-dir -r requirements.txt

RUN mkdir -p generated && \
    python -m grpc_tools.protoc \
    -I../atra-proto/proto \
    --python_out=./generated \
    --grpc_python_out=./generated \
    ../atra-proto/proto/orderbook.proto && \
    sed -i 's/import orderbook_pb2/import generated.orderbook_pb2/' generated/orderbook_pb2_grpc.py

ENV PYTHONPATH=/usr/src/app/cli

RUN touch generated/__init__.py

ENTRYPOINT ["./atra"]