# Internet of Plants Monitor Server

This is the main server all iot devices `embedded` talk to. It also informs and obeys `iop-monitor-client`. To display what is going on with the iot devices and manage them.

It produces data in json, and acts on POST/PUT/DELETE requests, updating the system accordingly. Each user has their account with their plants. The plants receive constant events, so you can keep track of the system. It currently is tied to a specific set of events that makes sense to narrow our study.

Create a new plant, open it in a new tab (click the card). Now hardcode your credentials (yes this is bad) in the embedded code and the `plant_id`, and it will sign itself up. We will have to think about a better way of signing up without plaint-text credentials or even as hardcoded data, ideally the plant signs itself up and stores its id in the EEPROM. The credentials should be a pre-generated token stored automatically on the embedded system somehow.

## Dependencies

Ubuntu (all devops scripts assume ubuntu), Postgres, rust (+ cargo)

## Setup local environment

*This scripts install postgresql, creates a database named iop and sets 'postgres' psql user's password to 'postgres' (only available at 127.0.0.1)*

`./tools/install-dependencies.sh`

You will need to change the client host to `http://localhost:3002`

## Setup production server for the first time

*Warning this is a destructive process, if you have something running in the server, check the scripts (they are fairly small)*

*This runs postgres in the same machine as the server, which is a dumb idea, but we are currently running on scarce resources, so it runs together, if you have more resources you can run them separately and write your own tools :)*

`./setup-server.sh example.com`

Replacing example.com with your domain name. This assumes you have SSH access.

**It must be a domain name, since we need it to setup HTTPs using let's encrypt**

If you provide an IP instead of a DNS domain a lot of things will break. Be warned.

**The domain name can be skipped if you define the DOMAIN env var in .env**

For example, in this git folder type (replacing example.com for the right domain):

`echo "DOMAIN=example.com" > tools/.env`

That will make all devops scripts default to this domain, so you won't have to type the same domain all the time. And since most devops operations happen targeting the same server (at our scale), this helps a lot.

Now you can just type:

`./setup-machine.sh`

or

`./tools/deploy.sh`

## Deploying it after the first time

Now you have the entire environment setup, you can just deploy the binary and the migrations (and update the devops scripts if needed). Type:

`./tools/deploy.sh example.com`

If you have set "DOMAIN=example.com" in `tools/.env`, you can avoid typing the domain.

## Contributing

Some decisions around the code are because of the low resources, we tend to cache most of what we can. That means some data may be a little stale. Just check the code, it's defined by a proc-macro on the target api function. It has a flush time that we consider appropriate, but if you find some bugs related to stale vales that may be it. So go check the api function for its cache timeout. And maybe even the `cache` proc macro (it allows stale data to be used while the new one is being generated instead of blocking other users).

## License

[GNU Affero General Public License version 3 or later (AGPL-3.0+)](https://github.com/internet-of-plants/iop-monitor-server/blob/master/LICENSE)
