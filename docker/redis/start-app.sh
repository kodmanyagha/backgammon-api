#!/usr/bin/env sh

rm /app/redis/data/valkey.sock 1> /dev/null 2>& 1

nohup /usr/local/bin/valkey-server /etc/valkey.conf \
  > /app/redis/data/nohup.out 2>&1 &

sleep 500

tail -f /app/redis/data/nohup.out
