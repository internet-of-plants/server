#!/usr/bin/env bash

#sudo apt-get -q -y --fix-missing update < /dev/null
#sudo apt-get -q -y --with-new-pkgs upgrade < /dev/null
#sudo apt-get -q -y autoremove < /dev/null

SCRIPTPATH="$(cd "$(dirname "$0")" >/dev/null 2>&1; pwd -P)"
echo $SCRIPTPATH
ulimit -Sn $(ulimit -Hn)

sudo certbot renew > /var/log/iop/certbot.log
screen -dmS monitor-iop /opt/iop/monitor.sh
screen -ls
