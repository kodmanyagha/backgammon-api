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
ENV MYSQL_ROOT_HOST="%"
ENV MYSQL_ALLOW_EMPTY_PASSWORD: 1


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

### INSTALLING MYSQL

#COPY mysql-8.4.7-linux-glibc2.28-x86_64.tar.xz /mysql.tar.xz
RUN wget https://downloads.mysql.com/archives/get/p/23/file/mysql-8.4.7-linux-glibc2.28-x86_64.tar.xz -O mysql.tar.xz

RUN tar -xf mysql.tar.xz -C /opt
RUN ln -s /opt/mysql-8.4.7-linux-glibc2.28-x86_64 /opt/mysql
RUN ln -s /opt/mysql/bin/* /usr/local/bin/

RUN mkdir -p /home/${DOCKER_USER}/mysql_data/log
RUN mkdir -p /home/${DOCKER_USER}/mysql_data/data
RUN mkdir -p /home/${DOCKER_USER}/mysql_data/tmp
RUN mkdir -p /home/${DOCKER_USER}/mysql_data/share/mysql
RUN mkdir -p /home/${DOCKER_USER}/mysql_data/lib/plugin
RUN chown -R ${DOCKER_USER}:${DOCKER_USER} /home/${DOCKER_USER}/mysql_data
COPY my.cnf /home/${DOCKER_USER}/.my.cnf
RUN chown ${DOCKER_USER}:${DOCKER_USER} /home/${DOCKER_USER}/.my.cnf

COPY start-app.sh /usr/local/bin/start-app.sh
RUN chmod +x /usr/local/bin/start-app.sh

# When you done clear apk cache.
RUN rm -rf /var/lib/apt/lists/*
RUN apt-get clean


####################### Normal user commands
USER ${DOCKER_USER}

WORKDIR /home/${DOCKER_USER}/mysql_data

RUN sed -i 's/DOCKER_USER/'"$DOCKER_USER"'/g' ~/.my.cnf

EXPOSE 3306

ENTRYPOINT ["/usr/bin/tini", "--"]

CMD /bin/bash /usr/local/bin/start-app.sh
