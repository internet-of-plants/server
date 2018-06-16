
# Installing

`curl https://sh.rustup.rs -sSf | sh`

*Ubuntu 14 doesn't have the libsodium-dev package, you have to add the ppa*
`sudo apt-get install -y gcc postgresql libsodium-dev`

`cargo install`

# Automatic Deploy

Set the same `AUTH_TOKEN` env variable in 'travis' and 'deploy' server to allow for automatic deploy

# Deploy

`git clone https://github.com/internet-of-plants/server.git`
`cd server`
`bash dependencies/setup`
`bash dependencies/deploy`
