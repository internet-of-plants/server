#!/usr/bin/env bash

sudo apt-get -q -y update < /dev/null
sudo apt-get -q -y install postgresql postgresql-contrib firejail libssl-dev < /dev/null;

sudo -i -u postgres psql -c "CREATE DATABASE iop;" postgres;
sudo -i -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres';" postgres;
