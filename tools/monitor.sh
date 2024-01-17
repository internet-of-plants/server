#!/usr/bin/env bash

SCRIPTPATH="$(cd "$(dirname "$0")" >/dev/null 2>&1; pwd -P)"
FOLDER=/tmp/iop-$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 13 ; echo '').root

for f in /tmp/iop-*.root; do
  umount -l $f/bin
  umount -l $f/usr
  umount -l $f/proc
  umount -l $f/lib
  umount -l $f/lib64
  umount -l $f/etc
  #umount -l $f/sbin
  #umount -l $f/var/lock
  #umount -l $f/var/tmp
  #umount -l $f/var/run
  #umount -l $f/tmp
  #umount -l $f/sys
  #umount -l $f/dev
  #umount -l $f/lib32
  #umount -l $f/libx32
  #umount -l $f/run
  #umount -l $f/boot
  rm -rf $f
done

for f in /tmp/iop-*.tmpfs; do
  rm -rf $f
done

# Allows to update the binary without stopping it, and jails it
echo $FOLDER
mkdir -p $FOLDER/migrations $FOLDER/packages $FOLDER/dev $FOLDER/etc $FOLDER/proc $FOLDER/tmp $FOLDER/var/tmp $FOLDER/var/lock $FOLDER/bin $FOLDER/sbin $FOLDER/sys $FOLDER/var/run $FOLDER/var/crash $FOLDER/usr $FOLDER/lib $FOLDER/run $FOLDER/home/iop $FOLDER/boot $FOLDER/lib32 $FOLDER/lib64 $FOLDER/libx32 $FOLDER/home/iop

ln /var/log/iop/monitor.log $FOLDER/monitor.log

mount --bind /bin $FOLDER/bin
mount --bind /lib $FOLDER/lib
mount --bind /lib64 $FOLDER/lib64
mount --bind /usr $FOLDER/usr
mount --bind /proc $FOLDER/proc
mount --bind /etc $FOLDER/etc

#mkdir -p $FOLDER.tmpfs $FOLDER.var.tmpfs
#mount --bind $FOLDER.tmpfs $FOLDER/tmp
#mount --bind $FOLDER.var.tmpfs $FOLDER/var/tmp
#mount --bind /var/lock $FOLDER/var/lock
#mount --bind /var/run $FOLDER/var/run
#mount --bind /sbin $FOLDER/sbin
#mount --bind /lib32 $FOLDER/lib32
#mount --bind /libx32 $FOLDER/libx32
#mount --bind /sys $FOLDER/sys
#mount --bind /dev $FOLDER/dev
#mount --bind /run $FOLDER/run
#mount --bind /boot $FOLDER/boot

#echo "root:x:0:0:root:/root:/bin/bash" > $FOLDER/etc/passwd
#echo "iop:x:1000:1000::/home/iop:/bin/sh" >> $FOLDER/etc/passwd
#echo "root	ALL=(ALL:ALL) ALL" >> $FOLDER/etc/sudoers
#echo "root:*:16176:0:99999:7:::" > $FOLDER/etc/shadow
#echo "iop:!:19176:0:99999:7:::" >> $FOLDER/etc/shadow
#cp -r /etc/ssl $FOLDER/etc/ssl
#cp -r /etc/pam.d $FOLDER/etc/pam.d
#cp -r /etc/pam.d $FOLDER/etc/pam.d

cp /var/run/utmp $FOLDER/var/run/utmp
cp $SCRIPTPATH/migrations/* $FOLDER/migrations/
cp -r $SCRIPTPATH/packages/* $FOLDER/packages/
cp $SCRIPTPATH/run-server-with-logging.sh $FOLDER/
cp /root/cert.pem $FOLDER/
cp /root/privkey.pem $FOLDER/
cp $SCRIPTPATH/server $FOLDER/
sudo chmod 100 $FOLDER/server
sudo chmod 500 $FOLDER/run-server-with-logging.sh
sudo chmod 400 $FOLDER/cert.pem
sudo chmod 400 $FOLDER/privkey.pem
sudo chmod 400 $FOLDER/migrations/*
sudo chmod 500 $FOLDER/migrations
sudo chmod 400 $FOLDER/packages/*
sudo chmod 500 $FOLDER/packages
sudo chmod 500 $FOLDER/packages/target_prototypes/*
sudo chmod 500 $FOLDER/packages/target_prototypes
sudo chmod 400 $FOLDER/packages/sensor_prototypes/*
sudo chmod 500 $FOLDER/packages/sensor_prototypes

for path in "$FOLDER/packages/target_prototypes/"*; do
  folder=$(basename "$path")
  sudo chmod 400 $FOLDER/packages/target_prototypes/$folder/$folder.json
  sudo chmod 500 $FOLDER/packages/target_prototypes/$folder/targets
  sudo chmod 400 $FOLDER/packages/target_prototypes/$folder/targets/*
done

sudo chmod 555 $FOLDER
sudo chmod 777 $FOLDER/tmp
sudo chown iop:iop $FOLDER/home/iop
sudo chown iop:iop $FOLDER/server
sudo chown iop:iop $FOLDER/run-server-with-logging.sh
sudo chown iop:iop $FOLDER/var/crash
sudo chown iop:iop $FOLDER/cert.pem
sudo chown iop:iop $FOLDER/privkey.pem
sudo chown iop:iop $FOLDER/migrations
sudo chown iop:iop $FOLDER/migrations/*
sudo chown iop:iop $FOLDER/packages
sudo chown iop:iop $FOLDER/packages/*
sudo chown iop:iop $FOLDER/packages/target_prototypes/*
sudo chown iop:iop $FOLDER/packages/sensor_prototypes/*
sudo chown root:iop $FOLDER

for path in "$FOLDER/packages/target_prototypes/"*; do
  folder=$(basename "$path")
  sudo chown iop:iop $FOLDER/packages/target_prototypes/$folder/*
  sudo chown iop:iop $FOLDER/packages/target_prototypes/$folder/targets/*
done

firejail --noprofile --private-tmp --chroot=$FOLDER << "EOT"
cd /
ls
pio -h
sudo -su iop ./run-server-with-logging.sh
EOT
