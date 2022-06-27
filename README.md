# Internet of Plants - Central Management Server

This is the central server that monitors and manages iot devices. Integrates with [internet-of-plants/iop](https://github.com/internet-of-plants/iop).

It can also be interfaced with [`internet-of-plants/client`]. It displays what is going on with the iot devices in real time and allows you to manage them at scale. While keeping them secure and up to date.

It produces data in json, and acts on POST/PUT/DELETE requests, updating the system accordingly. Each user has their account with their plants. The plants receive constant events, so you can keep track of the system. It currently is tied to a specific set of events that makes sense to narrow our study.

## Features

- 

## Dependencies

It should work on all GNU/Linux distributions, Windows devices and MacOS devices.

But all production focused devops scripts assume the user is on Ubuntu and has postgres, [rustc + cargo](https://rustup.rs) and [platformio cli](https://docs.platformio.org/en/latest/core/installation.html#installation-methods) installed.

## Setup local environment

*This scripts install postgresql, creates a database named iop and sets 'postgres' psql user's password to 'postgres' (only available at 127.0.0.1)*

`./tools/install-dependencies.sh`

# Deploy

Check [DEPLOY.md](https://github.com/internet-of-plants/blob/main)

## License

[GNU Affero General Public License version 3 or later (AGPL-3.0+)](https://github.com/internet-of-plants/server/blob/main/LICENSE.md)
