#!/usr/bin/env bash

DOMAIN=$1

if [ -z "$DOMAIN" ]; then
  echo "Please provide the domain to setup the https certificate for (through ssh)"
  echo "./setup-machine.sh example.com"

  echo ""
  read -p "Are you sure you want to continue without setting up a https certificate? [y/N] " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Ok, leaving"
    exit
  fi
fi

echo "Setup swapfile"
fallocate -l 2G /swapfile-iop
chmod 600 /swapfile-iop
mkswap /swapfile-iop
swapon /swapfile-iop
sysctl vm.swappiness=10
echo '/swapfile-iop none swap sw 0 0' | tee -a /etc/fstab

echo "Setup firewall"
ufw disable
ufw default deny
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 4001/tcp
ufw --force enable

echo "Install ubuntu dependencies needed"
apt-get -q -y update < /dev/null
apt-get -q -y install postgresql postgresql-contrib snap firejail libssl-dev binutils build-essential < /dev/null;

if [ ! -z "$DOMAIN" ]; then
  echo "Setup certbot"
  snap install core < /dev/null
  snap refresh core < /dev/null
  apt-get -q -y remove certbot < /dev/null
  snap install --classic certbot
  ln -s /snap/bin/certbot /usr/bin/certbot
  certbot certonly --standalone --register-unsafely-without-email --non-interactive --domain $DOMAIN --agree-tos
  ln -s /etc/letsencrypt/live/$DOMAIN/privkey.pem /root/privkey.pem
  ln -s /etc/letsencrypt/live/$DOMAIN/fullchain.pem /root/cert.pem

  CRON=$(crontab -l 2>/dev/null | grep '/opt/iop/renew-cert.sh')
  echo "$CRON"
  if [ -z "$CRON" ]; then
    echo "Add certbot to cron"
    (crontab -l 2>/dev/null; echo "52 0,12 * * * root /opt/iop/renew-cert.sh >> /var/iop/renew-cert.log 2>&1") | crontab -
  fi
fi

echo "Create postgresql credentials"
sudo -i -u postgres psql -c "CREATE DATABASE iop;" postgres;
sudo -i -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres';" postgres;

# Create user that will actually run the server
echo "Creating user iop"
useradd iop
mkdir -p /var/log/iop

echo "Setting filesystem permissions"
touch /var/log/iop/certbot.log
chmod 640 /var/log/iop/certbot.log
chown iop:root /var/log/iop/certbot.log

touch /var/log/iop/monitor.log
chmod 640 /var/log/iop/monitor.log
chown iop:root /var/log/iop/monitor.log

touch /var/log/iop/run-server.cron.log
chmod 640 /var/log/iop/run-server.cron.log
chown iop:root /var/log/iop/run-server.cron.log

chmod 770 /var/log/iop
chown iop:root /var/log/iop

mkdir -p /opt/iop

chmod 770 /opt/iop
chown iop:root /opt/iop

# Add run-server.sh to crontab to run on reboot
CRON=$(crontab -l 2>/dev/null | grep '/opt/iop/run-server.sh')
if [ -z "$CRON" ]; then
  echo "Add server to cron"
  (crontab -l 2>/dev/null; echo "@reboot /opt/iop/run-server.sh >> /var/log/iop/run-server.cron.log 2>&1") | crontab -
fi

# Reboot every night
CRON=$(crontab -l 2>/dev/null | grep '00 0 * * * root /usr/sbin/shutdown -r now')
if [ -z "$CRON" ]; then
  echo "Add daily update to cron"
  (crontab -l 2>/dev/null; echo "00 0 * * * root /usr/sbin/shutdown -r now") | crontab -
fi
