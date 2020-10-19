#!/usr/bin/env bash

SCRIPTPATH="$(cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P)"

DOMAIN=$1

if [ -z "$DOMAIN" ]; then
  source $SCRIPTPATH/.env
fi

if [ -z "$DOMAIN" ]; then
  echo "Please provide the domain to deploy to (through ssh)"
  echo "./deploy.sh example.com"
  exit
fi

cd $SCRIPTPATH/..
  cargo build --release
cd -

ssh root@$DOMAIN "mkdir -p /root/iop/migrations"
scp $SCRIPTPATH/monitor.sh root@$DOMAIN:/root/iop/
scp $SCRIPTPATH/renew-cert.sh root@$DOMAIN:/root/iop/
scp $SCRIPTPATH/run-server.sh root@$DOMAIN:/root/iop/
scp $SCRIPTPATH/../target/release/iop-monitor-server root@$DOMAIN:/root/iop/
scp $SCRIPTPATH/../migrations/* root@$DOMAIN:/root/iop/migrations/

ssh root@$DOMAIN "screen -S monitor-iop -X quit"
ssh root@$DOMAIN "/root/iop/run-server.sh >> /root/iop/run-server.cron.log 2>&1"
