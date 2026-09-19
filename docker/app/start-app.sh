#!/usr/bin/env sh

mkdir -p ~/.supervisord/log
mkdir -p ~/.supervisord/run

cp /etc/supervisord.ini.example ~/.supervisord/supervisord.conf

sed -i 's/DOCKER_USER/'"$DOCKER_USER"'/g' ~/.supervisord/supervisord.conf

exec /usr/bin/supervisord -c ~/.supervisord/supervisord.conf
