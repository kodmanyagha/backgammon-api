FROM ubuntu:24.04

ARG DOCKER_UID
ARG DOCKER_GID
ARG DOCKER_USER
ARG DOCKER_GROUP

ENV DOCKER_UID=${DOCKER_UID}
ENV DOCKER_GID=${DOCKER_GID}
ENV DOCKER_USER=${DOCKER_USER}
ENV DOCKER_GROUP=${DOCKER_GROUP}

RUN apt-get update
RUN apt-get upgrade -y
RUN DEBIAN_FRONTEND=noninteractive TZ=Europe/Istanbul \
      apt-get install -y wget curl ca-certificates tini xz-utils \
        libaio1t64 libaio-dev libnuma1 libnuma-dev \
        libncurses-dev libncurses6 ncurses-base \
        nano ntp tzdata

RUN ln -snf /usr/share/zoneinfo/Europe/Istanbul /etc/localtime && \
      echo "Europe/Istanbul" > /etc/timezone

RUN update-ca-certificates

RUN userdel -r ubuntu
RUN /usr/sbin/groupadd --gid ${DOCKER_GID} ${DOCKER_GROUP}
RUN /usr/sbin/useradd -m -d /home/${DOCKER_USER} -s /bin/bash --gid ${DOCKER_GID} --uid ${DOCKER_UID} ${DOCKER_USER}

RUN ln -s /usr/lib/x86_64-linux-gnu/libaio.so.1t64 /usr/lib/x86_64-linux-gnu/libaio.so.1

### INSTALLING VALKEY

RUN wget https://download.valkey.io/releases/valkey-9.0.3-noble-x86_64.tar.gz
RUN mkdir -p /home/${DOCKER_USER}/.tmp/valkey_files
RUN tar -xf valkey-9.0.3-noble-x86_64.tar.gz -C /home/${DOCKER_USER}/.tmp/valkey_files
RUN chmod +X /home/${DOCKER_USER}/.tmp/valkey_files/valkey-9.0.3-noble-x86_64/bin/*
RUN mv /home/${DOCKER_USER}/.tmp/valkey_files/valkey-9.0.3-noble-x86_64/bin/* /usr/local/bin
RUN rm -rf /home/${DOCKER_USER}/.tmp/valkey_files
RUN rm /valkey-9.0.3-noble-x86_64.tar.gz

RUN mkdir -p /app/redis/data
COPY valkey.conf /etc/valkey.conf
RUN sed -i 's/DOCKER_USER/'"$DOCKER_USER"'/g' /etc/valkey.conf

RUN chown -R ${DOCKER_USER}:${DOCKER_USER} /app
RUN chmod -R 775 /app

COPY start-app.sh /usr/local/bin/start-app.sh
RUN chmod +x /usr/local/bin/start-app.sh

RUN apt-get clean


####################### Normal user commands
USER ${DOCKER_USER}

WORKDIR /app/redis/data

EXPOSE 6379

ENTRYPOINT ["/usr/bin/tini", "--"]

CMD /bin/sh /usr/local/bin/start-app.sh
