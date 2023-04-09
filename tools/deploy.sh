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
  RUSTFLAGS=-g cargo build --release
  if [ $? -ne 0 ]; then
    cd -
    echo "Build failed, bailing out"
    exit
  fi
cd -

scp $SCRIPTPATH/monitor.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 120 /opt/iop/monitor.sh"
ssh root@$DOMAIN "chown iop.root /opt/iop/monitor.sh"

scp $SCRIPTPATH/renew-cert.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 120 /opt/iop/renew-cert.sh"
ssh root@$DOMAIN "chown iop.root /opt/iop/renew-cert.sh"

scp $SCRIPTPATH/run-server.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 120 /opt/iop/run-server.sh"
ssh root@$DOMAIN "chown iop.root /opt/iop/run-server.sh"

scp $SCRIPTPATH/run-server-with-logging.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 120 /opt/iop/run-server-with-logging.sh"
ssh root@$DOMAIN "chown iop.root /opt/iop/run-server-with-logging.sh"

ssh root@$DOMAIN "mkdir -p /opt/iop/migrations"
ssh root@$DOMAIN "chmod 570 /opt/iop/migrations"
ssh root@$DOMAIN "chown iop.root /opt/iop/migrations"

scp -r $SCRIPTPATH/../migrations/* root@$DOMAIN:/opt/iop/migrations/
ssh root@$DOMAIN "chmod 420 /opt/iop/migrations/*"
ssh root@$DOMAIN "chown iop.root /opt/iop/migrations/*"

ssh root@$DOMAIN "mkdir -p /opt/iop/packages"
ssh root@$DOMAIN "chmod 420 /opt/iop/packages"
ssh root@$DOMAIN "chown iop.root /opt/iop/packages"

ssh root@$DOMAIN "mkdir -p /opt/iop/packages/sensor_prototypes"
ssh root@$DOMAIN "chmod 570 /opt/iop/packages/sensor_prototypes"
ssh root@$DOMAIN "chown iop.root /opt/iop/packages/sensor_prototypes"

if [ -d "$SCRIPTPATH/../packages/sensor_prototypes" ]; then
  scp $SCRIPTPATH/../packages/sensor_prototypes/* root@$DOMAIN:/opt/iop/packages/sensor_prototypes/
  ssh root@$DOMAIN "chmod 420 /opt/iop/packages/sensor_prototypes/*"
  ssh root@$DOMAIN "chown iop.root /opt/iop/packages/sensor_prototypes/*"
fi

ssh root@$DOMAIN "mkdir -p /opt/iop/packages/target_prototypes"
ssh root@$DOMAIN "chmod 570 /opt/iop/packages/target_prototypes"
ssh root@$DOMAIN "chown iop.root /opt/iop/packages/target_prototypes"

for path in "$SCRIPTPATH/../packages/target_prototypes/"*; do
  folder=$(basename "$path")
  ssh root@$DOMAIN "mkdir -p /opt/iop/packages/target_prototypes/$folder/targets"
  ssh root@$DOMAIN "chmod 570 /opt/iop/packages/target_prototypes/$folder"
  ssh root@$DOMAIN "chown iop.root /opt/iop/packages/target_prototypes/$folder"
  ssh root@$DOMAIN "chmod 570 /opt/iop/packages/target_prototypes/$folder/targets"
  ssh root@$DOMAIN "chown iop.root /opt/iop/packages/target_prototypes/$folder/targets"

  scp $path/$folder.json root@$DOMAIN:/opt/iop/packages/target_prototypes/$folder/
  ssh root@$DOMAIN "chmod 420 /opt/iop/packages/target_prototypes/$folder/$folder.json"
  ssh root@$DOMAIN "chown iop.root /opt/iop/packages/target_prototypes/$folder/$folder.json"

  scp $path/targets/* root@$DOMAIN:/opt/iop/packages/target_prototypes/$folder/targets/
  ssh root@$DOMAIN "chmod 420 /opt/iop/packages/target_prototypes/$folder/targets/*"
  ssh root@$DOMAIN "chown iop.root /opt/iop/packages/target_prototypes/$folder/targets/*"
done

scp $SCRIPTPATH/../target/release/server-bin root@$DOMAIN:/opt/iop/server
ssh root@$DOMAIN "chmod 120 /opt/iop/server"
ssh root@$DOMAIN "chown iop.root /opt/iop/server"

ssh root@$DOMAIN "reboot"
