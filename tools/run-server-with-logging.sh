#!/usr/bin/env bash

python3 -c "$(curl -fsSL https://raw.githubusercontent.com/platformio/platformio/master/scripts/get-platformio.py)"
export PATH=$PATH:/home/iop/.platformio/penv/bin
whoami
./server 2>&1 | tee -a ./monitor.log
