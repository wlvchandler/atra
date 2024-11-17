FROM python:3.10-slim

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

COPY kobara-proto/proto /usr/src/app/kobara-proto/proto

COPY kobara-cli /usr/src/app/kobara-cli

WORKDIR /usr/src/app/kobara-cli

RUN pip install --no-cache-dir -r requirements.txt

RUN chmod +x ./cli.py

ENTRYPOINT ["./cli.py"]