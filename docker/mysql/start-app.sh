#!/usr/bin/env sh

if [ ! -f "/home/$DOCKER_USER/mysql_data/data/ibdata1" ]; then
  echo "ibdata1 not found, initializing mysql data folder."

  /opt/mysql/bin/mysqld --initialize-insecure --basedir=/opt/mysql --datadir=/home/$DOCKER_USER/mysql_data/data
  sleep 2
fi

echo "mysql starting"
rm /home/$DOCKER_USER/mysql_data/data/mysqld.pid
rm /home/$DOCKER_USER/mysql_data/data/mysqld.sock.lock

nohup /opt/mysql/bin/mysqld --defaults-file=/home/$DOCKER_USER/.my.cnf \
    --user=$DOCKER_USER \
    > /home/$DOCKER_USER/mysql_data/data/nohup.out 2>&1 &

sleep 10

/opt/mysql/bin/mysql -u root <<EOSQL
DROP USER IF EXISTS '$MYSQL_USER'@'%';
DROP USER IF EXISTS '$MYSQL_USER'@'localhost';
DROP USER IF EXISTS 'root'@'%';


CREATE USER IF NOT EXISTS '$MYSQL_USER'@'%'
    IDENTIFIED WITH auth_socket BY '$MYSQL_PASSWORD' PASSWORD EXPIRE NEVER;

CREATE USER IF NOT EXISTS '$MYSQL_USER'@'localhost'
    IDENTIFIED WITH mysql_native_password BY '$MYSQL_PASSWORD';

CREATE USER IF NOT EXISTS 'root'@'%'
    IDENTIFIED WITH mysql_native_password BY '$MYSQL_PASSWORD' PASSWORD EXPIRE NEVER;


ALTER USER '$MYSQL_USER'@'%' DEFAULT ROLE ALL;
GRANT ALL PRIVILEGES ON *.* TO '$MYSQL_USER'@'%' WITH GRANT OPTION;

ALTER USER '$MYSQL_USER'@'localhost' DEFAULT ROLE ALL;
GRANT ALL PRIVILEGES ON *.* TO '$MYSQL_USER'@'localhost' WITH GRANT OPTION;


CREATE DATABASE IF NOT EXISTS $MYSQL_DATABASE;
CREATE DATABASE IF NOT EXISTS testing;
CREATE DATABASE IF NOT EXISTS db_export_temp;


GRANT ALL PRIVILEGES ON *.* TO '$MYSQL_USER'@'%';
GRANT ALL PRIVILEGES ON *.* TO '$MYSQL_USER'@'localhost';
GRANT ALL PRIVILEGES ON *.* TO 'root'@'%';

FLUSH PRIVILEGES;

EOSQL

tail -f /home/$DOCKER_USER/mysql_data/data/nohup.out
