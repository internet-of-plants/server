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

echo "Sending tools/monitor.sh"
scp $SCRIPTPATH/monitor.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 100 /opt/iop/monitor.sh"
ssh root@$DOMAIN "chown iop:root /opt/iop/monitor.sh"

echo "Sending tools/renew-cert.sh"
scp $SCRIPTPATH/renew-cert.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 100 /opt/iop/renew-cert.sh"
ssh root@$DOMAIN "chown iop:root /opt/iop/renew-cert.sh"

echo "Sending tools/run-server.sh"
scp $SCRIPTPATH/run-server.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 100 /opt/iop/run-server.sh"
ssh root@$DOMAIN "chown iop:root /opt/iop/run-server.sh"

echo "Sending tools/firejail.config"
scp $SCRIPTPATH/firejail.config root@$DOMAIN:/etc/firejail/
ssh root@$DOMAIN "chmod 100 /opt/iop/firejail.config"
ssh root@$DOMAIN "chown root:root /etc/firejail/firejail.config"

echo "Sending tools/run-server-with-logging.sh"
scp $SCRIPTPATH/run-server-with-logging.sh root@$DOMAIN:/opt/iop/
ssh root@$DOMAIN "chmod 100 /opt/iop/run-server-with-logging.sh"
ssh root@$DOMAIN "chown iop:root /opt/iop/run-server-with-logging.sh"

echo "Sending migrations"
ssh root@$DOMAIN "mkdir -p /opt/iop/migrations"
ssh root@$DOMAIN "chmod 400 /opt/iop/migrations"
ssh root@$DOMAIN "chown iop:root /opt/iop/migrations"

echo "Sending migrations/*"
scp -r $SCRIPTPATH/../migrations/* root@$DOMAIN:/opt/iop/migrations/
ssh root@$DOMAIN "chmod 400 /opt/iop/migrations/*"
ssh root@$DOMAIN "chown iop:root /opt/iop/migrations/*"

echo "Sending packages"
ssh root@$DOMAIN "mkdir -p /opt/iop/packages"
ssh root@$DOMAIN "chmod 400 /opt/iop/packages"
ssh root@$DOMAIN "chown iop:root /opt/iop/packages"

echo "Sending packages/sensor_prototypes"
ssh root@$DOMAIN "mkdir -p /opt/iop/packages/sensor_prototypes"
ssh root@$DOMAIN "chmod 400 /opt/iop/packages/sensor_prototypes"
ssh root@$DOMAIN "chown iop:root /opt/iop/packages/sensor_prototypes"

if [ -d "$SCRIPTPATH/../packages/sensor_prototypes" ]; then
  echo "Sending packages/sensor_prototypes/*"
  scp $SCRIPTPATH/../packages/sensor_prototypes/* root@$DOMAIN:/opt/iop/packages/sensor_prototypes/
  ssh root@$DOMAIN "chmod 400 /opt/iop/packages/sensor_prototypes/*"
  ssh root@$DOMAIN "chown iop:root /opt/iop/packages/sensor_prototypes/*"
fi

echo "Sending packages/target_prototypes"
ssh root@$DOMAIN "mkdir -p /opt/iop/packages/target_prototypes"
ssh root@$DOMAIN "chmod 400 /opt/iop/packages/target_prototypes"
ssh root@$DOMAIN "chown iop:root /opt/iop/packages/target_prototypes"

for path in "$SCRIPTPATH/../packages/target_prototypes/"*; do
  folder=$(basename "$path")
  echo "Sending packages/target_prototypes/$folder/targets"
  ssh root@$DOMAIN "mkdir -p /opt/iop/packages/target_prototypes/$folder/targets"
  ssh root@$DOMAIN "chmod 400 /opt/iop/packages/target_prototypes/$folder"
  ssh root@$DOMAIN "chown iop:root /opt/iop/packages/target_prototypes/$folder"
  ssh root@$DOMAIN "chmod 400 /opt/iop/packages/target_prototypes/$folder/targets"
  ssh root@$DOMAIN "chown iop:root /opt/iop/packages/target_prototypes/$folder/targets"

  echo "Sending packages/target_prototypes/$folder/$folder.json"
  scp $path/$folder.json root@$DOMAIN:/opt/iop/packages/target_prototypes/$folder/
  ssh root@$DOMAIN "chmod 400 /opt/iop/packages/target_prototypes/$folder/$folder.json"
  ssh root@$DOMAIN "chown iop:root /opt/iop/packages/target_prototypes/$folder/$folder.json"

  echo "Sending packages/target_prototypes/$folder/targets/*"
  scp $path/targets/* root@$DOMAIN:/opt/iop/packages/target_prototypes/$folder/targets/
  ssh root@$DOMAIN "chmod 400 /opt/iop/packages/target_prototypes/$folder/targets/*"
  ssh root@$DOMAIN "chown iop:root /opt/iop/packages/target_prototypes/$folder/targets/*"
done

echo "Sending server-bin"
scp $SCRIPTPATH/../target/release/server-bin root@$DOMAIN:/opt/iop/server
ssh root@$DOMAIN "chmod 100 /opt/iop/server"
ssh root@$DOMAIN "chown iop:root /opt/iop/server"

echo "screen kill"
ssh root@$DOMAIN "screen -X -S monitor-iop quit"

echo "process start"
ssh root@$DOMAIN "/opt/iop/run-server.sh"
