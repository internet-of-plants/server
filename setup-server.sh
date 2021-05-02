#!/usr/bin/env bash

DOMAIN=$1

SCRIPTPATH="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
if [ -z "$DOMAIN" ]; then
  source $SCRIPTPATH/tools/.env
fi


if [ -z "$DOMAIN" ]; then
  echo "Please provide the domain name to setup the server onto (through ssh)"
  echo "It must be a domain name, since we make a HTTPs cert for it (otherwise things will break)"
  echo "./setup-server.sh example.com"
  exit
fi

echo "Deploying to domain: $DOMAIN"

#PUBKEY=$(ssh -o 'StrictHostKeyChecking no' root@$DOMAIN cat /etc/ssh/ssh_host_dsa_key.pub)
#PUBKEY_IS_THERE=$(cat ~/.ssh/known_hosts | grep "$PUBKEY")
#if [ -z "$PUBKEY_IS_THERE" ]; then
#  echo "Adding pubkey to known hosts"
#  echo $PUBKEY >> ~/.ssh/known_hosts
#fi

# Setup dependencies and database
ssh root@$DOMAIN "bash -s" < $SCRIPTPATH/tools/setup-machine.sh $DOMAIN

# Deploy actual server
$SCRIPTPATH/tools/deploy.sh $DOMAIN
