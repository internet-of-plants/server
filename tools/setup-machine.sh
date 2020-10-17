#!/usr/bin/env bash

DOMAIN=$1

if [ -z "$DOMAIN" ]; then
  echo "Please provide the domain to setup the https certificate for (through ssh)"
  echo "./setup-machine.sh example.com"

  echo ""
  read -r -p "Are you sure you want to continue without setting up a https certificate? [y/N] " RESPONSE
  case "$RESPONSE" in [yY]);;
    *); exit;;
  esac
fi

sudo sysctl vm.swappiness=10

# Setup swapfile
sudo fallocate -l 5G /swapfile-iop
sudo chmod 600 /swapfile-iop
sudo mkswap /swapfile-iop
sudo swapon /swapfile-iop
echo '/swapfile-iop none swap sw 0 0' | sudo tee -a /etc/fstab

# Setup firewall
ufw disable
ufw default deny
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 4000/tcp # this is just so we don't break my other stuff running in the same server
ufw allow 4001/tcp
ufw --force enable

sudo apt-get -q -y update < /dev/null
sudo apt-get -q -y install postgresql postgresql-contrib snap < /dev/null;

# Setup certbot
if [ !-z "$DOMAIN" ]; then
  sudo snap install core < /dev/null
  sudo snap refresh core < /dev/null
  sudo apt-get -q -y remove certbot < /dev/null
  sudo snap install --classic certbot
  sudo ln -s /snap/bin/certbot /usr/bin/certbot
  sudo certbot certonly --standalone --register-unsafely-without-email --non-interactive --domain $DOMAIN --agree-tos
  sudo ln -s /etc/letsencrypt/live/$DOMAIN/privkey.pem /root/iop/privkey.pem
  sudo ln -s /etc/letsencrypt/live/$DOMAIN/cert.pem /root/iop/cert.pem

  # Test it
  sudo certbot renew --dry-run

  CRON=$(crontab -l 2>/dev/null | grep '/root/iop/renew-cert.sh')
  echo "$CRON"
  if [ -z "$CRON" ]; then
  	echo "Add certbot to cron"
  	(crontab -l 2>/dev/null; echo "52 0,12 * * * root /root/iop/renew-cert.sh") | crontab -
  fi
fi

sudo -i -u postgres psql -c "CREATE DATABASE iop;" postgres;
sudo -i -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres';" postgres;

# Add run-server.sh to crontab to run on reboot
CRON=$(crontab -l 2>/dev/null | grep '/root/iop/run-server.cron.log')
echo "$CRON"
if [ -z "$CRON" ]; then
	echo "Add server to cron"
	(crontab -l 2>/dev/null; echo "@reboot /root/iop/run-server.sh >> /root/iop/run-server.cron.log 2>&1") | crontab -
fi
