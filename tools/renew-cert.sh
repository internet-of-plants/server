#!/usr/bin/env bash

sudo /bin/certbot renew > /root/iop/certbot.log
cat /root/iop/certbot.log | grep "No renewals were attempted."
if [ "$?" -eq "0" ]; then
  screen -S monitor-iop -X quit
fi

/root/iop/run-server.sh >> /root/iop/run-server.cron.log 2>&1
