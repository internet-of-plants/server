#!/usr/bin/env bash

SCRIPTPATH="$(cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P)"
FOLDER=/tmp/$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 13 ; echo '')

# Clear tmp folder
find /tmp -type f -atime +5 -delete

# Allows to update the binary without stopping it, and jails it
echo $FOLDER
mkdir -p $FOLDER/migrations

cp $SCRIPTPATH/migrations/* $FOLDER/migrations/
cp /root/cert.pem $FOLDER/
cp /root/privkey.pem $FOLDER/
cp $SCRIPTPATH/iop-monitor-server $FOLDER/
sudo chmod 500 $FOLDER/iop-monitor-server
sudo chmod 400 $FOLDER/cert.pem
sudo chmod 400 $FOLDER/privkey.pem
sudo chmod 400 $FOLDER/migrations/*
sudo chmod 500 $FOLDER/migrations
sudo chmod 500 $FOLDER
sudo chown iop.iop $FOLDER/iop-monitor-server
sudo chown iop.iop $FOLDER/cert.pem
sudo chown iop.iop $FOLDER/privkey.pem
sudo chown iop.iop $FOLDER/migrations/*
sudo chown iop.iop $FOLDER/migrations
sudo chown iop.iop $FOLDER

#mkdir -p $FOLDER/etc
#cp /etc/passwd $FOLDER/etc/
#sudo chmod 440 $FOLDER/etc/*
#sudo chmod 550 $FOLDER/etc
#sudo chown iop.root $FOLDER/etc/*
#sudo chown iop.root $FOLDER/etc
#
#mkdir -p $FOLDER/bin
#cp -r /bin/* $FOLDER/bin/
#cp /bin/sh $FOLDER/bin/
#cp /bin/bash $FOLDER/bin/
#cp /bin/sudo $FOLDER/bin/
#cp /bin/tee $FOLDER/bin/
#sudo chmod 110 $FOLDER/bin/*
#sudo chmod 550 $FOLDER/bin
#sudo chown iop.root $FOLDER/bin/*
#sudo chown iop.root $FOLDER/bin
#
#mkdir -p $FOLDER/lib/x86_64-linux-gnu
#cp -ra /lib/* $FOLDER/lib/*
#cp /lib/x86_64-linux-gnu/libtinfo.so.6 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/libdl.so.2 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/libc.so.6 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/libpthread.so.0 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/libpcre2-8.so.0 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/libcap-ng.so.0 $FOLDER/lib/x86_64-linux-gnu/
#cp /usr/lib/sudo/libsudo_util.so.0 $FOLDER/lib/
#cp /lib/x86_64-linux-gnu/libutil.so.1 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/libselinux.so.1 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/libaudit.so.1 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/libaudit.so.1 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/librt.so.1 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/libgcc_s.so.1 $FOLDER/lib/x86_64-linux-gnu/
#cp /lib/x86_64-linux-gnu/libm.so.6 $FOLDER/lib/x86_64-linux-gnu/
#sudo chmod 110 $FOLDER/lib/*
#sudo chmod 550 $FOLDER/lib
#sudo chown iop.root $FOLDER/lib/*
#sudo chown iop.root $FOLDER/lib
#
#mkdir -p $FOLDER/lib64
#cp -ra /lib64/* $FOLDER/lib64/
#cp /lib64/ld-linux-x86-64.so.2 $FOLDER/lib64/
#sudo chmod 110 $FOLDER/lib64/*
#sudo chmod 550 $FOLDER/lib64
#sudo chown iop.root $FOLDER/lib64/*
#sudo chown iop.root $FOLDER/lib64
#
#sudo chown iop.root $FOLDER
#sudo chmod 550 $FOLDER
#
#id
#
#cd $FOLDER
#sudo chroot $FOLDER /bin/bash << "EOT"
#id
#sudo -su iop
#id
#$FOLDER/iop-monitor-server 2>&1 | tee -a $1
#EOT
cd $FOLDER
sudo su iop << "EOT"
whoami
ls
./iop-monitor-server 2>&1 | tee -a $1
EOT
