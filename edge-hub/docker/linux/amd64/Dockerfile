﻿ARG base_tag=6.0-alpine

FROM mcr.microsoft.com/dotnet/aspnet:${base_tag}

ARG EXE_DIR=.

ADD ./watchdog/x86_64-unknown-linux-musl/release/watchdog /usr/local/bin/watchdog
ADD ./mqtt/x86_64-unknown-linux-musl/release/mqttd /usr/local/bin/mqttd
ADD ./mqtt/broker.json /app/mqttd/broker.json

# RocksDB requires snappy	
RUN apk update && \	
    apk add --no-cache snappy libcap	

# Add an unprivileged user account for running Edge Hub	
# value was chosen as a large value to avoid a typical regular uid
ARG EDGEHUBUSER_ID	
ENV EDGEHUBUSER_ID ${EDGEHUBUSER_ID:-13623}
RUN adduser -Ds /bin/sh -u ${EDGEHUBUSER_ID} edgehubuser 	

# Add the CAP_NET_BIND_SERVICE capability to the dotnet binary because	
# we are starting Edge Hub as a non-root user	
RUN setcap 'cap_net_bind_service=+ep' /usr/share/dotnet/dotnet	

# Install RocksDB	
COPY $EXE_DIR/librocksdb/librocksdb.so.amd64 /usr/local/lib/librocksdb.so

WORKDIR /app

COPY Microsoft.Azure.Devices.Edge.Hub.Service/ ./
COPY $EXE_DIR/Microsoft.Azure.Devices.Edge.Hub.Service/hubStart.sh ./

# Expose MQTT, AMQP and HTTPS ports	
EXPOSE 1883/tcp
EXPOSE 8883/tcp	
EXPOSE 5671/tcp	
EXPOSE 443/tcp	

CMD echo "$(date --utc +"%Y-%m-%d %H:%M:%S %:z") Starting Edge Hub" && \
    exec /app/hubStart.sh
