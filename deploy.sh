#!/bin/bash

git pull origin main

kill $(pgrep -f tavla_api)

sea-orm-cli migrate

cargo build --release

./target/release/seed

(nohup ./target/release/tavla_api 2>&1) >> logs/tavla_api.log &

sleep 3

rm /opt/sock/tavla_api.sock
ln $(pwd)/server.sock /opt/sock/tavla_api.sock
chown $(whoami):www-data /opt/sock/tavla_api.sock
