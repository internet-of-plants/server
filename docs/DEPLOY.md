
# Setup production server for the first time

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
