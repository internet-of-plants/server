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
  if [ $? -ne 0 ]; then
    cd -
    echo "Build failed, bailing out"
    exit
  fi
cd -

ssh root@$DOMAIN "mkdir -p /opt/iop/migrations"
scp $SCRIPTPATH/monitor.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 120 /opt/iop/monitor.sh"
ssh root@$DOMAIN "chown iop.root /opt/iop/monitor.sh"

scp $SCRIPTPATH/renew-cert.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 120 /opt/iop/renew-cert.sh"
ssh root@$DOMAIN "chown iop.root /opt/iop/renew-cert.sh"

scp $SCRIPTPATH/run-server.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 120 /opt/iop/run-server.sh"
ssh root@$DOMAIN "chown iop.root /opt/iop/run-server.sh"

scp $SCRIPTPATH/../target/release/server root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 120 /opt/iop/server"
ssh root@$DOMAIN "chown iop.root /opt/iop/server"

ssh root@$DOMAIN "mkdir -p /opt/iop/migrations"
ssh root@$DOMAIN "chmod 570 /opt/iop/migrations"
ssh root@$DOMAIN "chown iop.root /opt/iop/migrations"

scp $SCRIPTPATH/../migrations/* root@$DOMAIN:/opt/iop/migrations/
ssh root@$DOMAIN "chmod 420 /opt/iop/migrations/*"
ssh root@$DOMAIN "chown iop.root /opt/iop/migrations/*"

ssh root@$DOMAIN "screen -S monitor-iop -X quit"
ssh root@$DOMAIN "screen -S startiopserver -X quit"

CRON=$(ssh root@$DOMAIN "crontab -l | grep /opt/iop/run-server.sh")
COMMAND=${CRON#@reboot }
ssh root@$DOMAIN "screen -dmS startiopserver $COMMAND"
