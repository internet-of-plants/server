#!/usr/bin/env bash

sudo apt-get -q -y --fix-missing update < /dev/null
sudo apt-get -q -y --with-new-pkgs upgrade < /dev/null
sudo apt-get -q -y autoremove < /dev/null

SCRIPTPATH="$(cd "$(dirname "$0")" > /dev/null 2>&1; pwd -P)"

sudo certbot renew > /var/log/iop/certbot.log
cat /var/log/iop/certbot.log | grep "No renewals were attempted."
if [ "$?" -eq "0" ]; then
  screen -S monitor-iop -X quit
  screen -S startiopserver -X quit

  CRON=$(ssh root@$DOMAIN "crontab -l | grep /opt/iop/run-server.sh")
  COMMAND=${CRON#@reboot }
  ssh root@$DOMAIN "screen -dmS startiopserver $COMMAND"

  # Some other stuff running in the same server, although unrelated
  screen -S monitor -X quit
  screen -S startserver -X quit
  CRON=$(ssh root@$DOMAIN "crontab -l | grep /root/run-server.sh")
  COMMAND=${CRON#@reboot }
  ssh root@$DOMAIN "screen -dmS startserver $COMMAND"
fi
