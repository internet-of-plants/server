#!/usr/bin/env bash

SCRIPTPATH="$(cd "$(dirname "$0")" >/dev/null 2>&1; pwd -P)"
FOLDER=/tmp/iop-$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 13 ; echo '')

rm -rf /tmp/iop-*

# Allows to update the binary without stopping it, and jails it
echo $FOLDER
mkdir -p $FOLDER/migrations

cp $SCRIPTPATH/migrations/* $FOLDER/migrations/
cp /root/cert.pem $FOLDER/
cp /root/privkey.pem $FOLDER/
cp $SCRIPTPATH/server $FOLDER/
sudo chmod 100 $FOLDER/server
sudo chmod 400 $FOLDER/cert.pem
sudo chmod 400 $FOLDER/privkey.pem
sudo chmod 400 $FOLDER/migrations/*
sudo chmod 500 $FOLDER/migrations
sudo chmod 555 $FOLDER
sudo chown iop.iop $FOLDER/server
sudo chown iop.iop $FOLDER/cert.pem
sudo chown iop.iop $FOLDER/privkey.pem
sudo chown iop.iop $FOLDER/migrations/*
sudo chown iop.iop $FOLDER/migrations
sudo chown root.iop $FOLDER

cd $FOLDER
firejail --noprofile --private=$FOLDER << "EOT"
sudo -su iop
whoami
./server 2>&1 | tee -a /var/log/iop/monitor.log
EOT
