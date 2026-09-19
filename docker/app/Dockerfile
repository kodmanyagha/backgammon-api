FROM ubuntu:24.04

ARG DOCKER_UID
ARG DOCKER_GID
ARG DOCKER_USER
ARG DOCKER_GROUP
ARG MYSQL_DATABASE
ARG MYSQL_USER
ARG MYSQL_PASSWORD

ENV DOCKER_UID=${DOCKER_UID}
ENV DOCKER_GID=${DOCKER_GID}
ENV DOCKER_USER=${DOCKER_USER}
ENV DOCKER_GROUP=${DOCKER_GROUP}
ENV MYSQL_DATABASE=${MYSQL_DATABASE}
ENV MYSQL_USER=${MYSQL_USER}
ENV MYSQL_PASSWORD=${MYSQL_PASSWORD}
ENV MYSQL_ROOT_PASSWORD=${MYSQL_PASSWORD}

RUN apt-get update
RUN apt-get upgrade -y
RUN apt-get install -y wget curl ca-certificates tini
RUN update-ca-certificates

RUN addgroup --gid ${DOCKER_GID} ${DOCKER_GROUP}
RUN adduser --disabled-password --shell /bin/bash --gid ${DOCKER_GID} -u ${DOCKER_UID} ${DOCKER_USER}

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

RUN wget -qO watchexec.tar.xz https://github.com/watchexec/watchexec/releases/latest/download/watchexec-2.3.3-x86_64-unknown-linux-gnu.tar.xz
RUN tar xf watchexec.tar.xz --strip-components=1 -C watchexec-temp
RUN mv watchexec-temp/watchexec /usr/local/bin
RUN chmod +x /usr/local/bin/watchexec

RUN apt-get install supervisor

COPY supervisord.ini.example /etc/supervisord.ini.example
COPY start-app.sh /usr/local/bin/start-app.sh
RUN chmod +x /usr/local/bin/start-app.sh

# When you done clear apk cache.
RUN apt-get clean

USER ${DOCKER_USER}

RUN mkdir -p /home/${DOCKER_USER}/app
WORKDIR /home/${DOCKER_USER}/app

#EXPOSE 8080
#EXPOSE 8081
#EXPOSE 8082

ENTRYPOINT ["/usr/bin/tini", "--"]

CMD /bin/sh /usr/local/bin/start-app.sh
