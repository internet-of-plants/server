#!/usr/bin/env bash

if [ -z "$1" ]; then
  echo "Please provide an existing log file that we have access to"
  echo "ex: ./run.sh /var/log/monitor.log"
  exit
fi

rm -rf /tmp/iop-*

SCRIPTPATH="$(cd "$(dirname "$0")" >/dev/null 2>&1; pwd -P)"
FOLDER=/tmp/iop-$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 13; echo '')

# Allows to update the binary without stopping it, and jails it
echo $FOLDER
mkdir -p $FOLDER/migrations

cargo build
if [ $? -eq 0 ]; then
  echo "Build failed, bailing out"
  exit
fi
cp $SCRIPTPATH/target/debug/server $FOLDER/
cp $SCRIPTPATH/migrations/* $FOLDER/migrations/

ln -s $1 $FOLDER/run.log

cd $FOLDER
firejail --noprofile --private=$FOLDER << "EOT"
sudo -su iop
whoami
./server 2>&1 | tee -a run.log
EOT
