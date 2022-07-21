#!/usr/bin/env bash

echo "deb http://security.ubuntu.com/ubuntu impish-security main" | sudo tee /etc/apt/sources.list.d/impish-security.list
sudo apt-get -q -y update < /dev/null
sudo apt-get -q -y install postgresql postgresql-contrib firejail libssl-dev libssl1.1 < /dev/null;

sudo -i -u postgres psql -c "CREATE DATABASE iop;" postgres;
sudo -i -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres';" postgres;
