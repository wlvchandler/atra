FROM elixir:1.15-alpine

WORKDIR /app

RUN apk add --no-cache build-base git protoc protobuf protobuf-dev netcat-openbsd
RUN mix local.hex --force && mix local.rebar --force

COPY atra-gateway/mix.exs atra-gateway/mix.lock ./

RUN mix deps.get

RUN mix escript.install hex protobuf --force
ENV PATH="${PATH}:/root/.mix/escripts"

RUN mkdir -p /atra-proto/proto
COPY atra-proto/proto/ /atra-proto/proto/

RUN mkdir -p _build/dev/lib/google_protos/priv/protos/google/protobuf
RUN cp -r /usr/include/google/protobuf/* _build/dev/lib/google_protos/priv/protos/google/protobuf/

COPY atra-gateway ./

RUN mix proto.compile && mix compile

CMD ["mix", "run", "--no-halt"]