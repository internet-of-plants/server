#!/usr/bin/env bash

SCRIPTPATH="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
FOLDER=/tmp/$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 13 ; echo '')

# Allows to update the binary without stopping it
echo $FOLDER
mkdir $FOLDER -p
cp $SCRIPTPATH/iop-monitor-server $FOLDER/monitor
$FOLDER/monitor 2>&1 | tee -a $1
