
## Install all necessary dependencies.
apt-get update
apt-get -y install curl git build-essential 

# Install Rust compiler
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
PATH="$HOME/.cargo/bin:$PATH"

# Get aries sources
git clone -b ipc-2023 https://github.com/plaans/aries.git aries

## build planner
cd aries
cargo build --release --bin lcp
cd ..

# Save executable
cp aries/target/release/lcp /aries-planner

## remove files not needed to execute the planner
rm -rf aries/
rm -rf $HOME/.cargo
apt-get -y purge curl git build-essential 
apt-get -y autoremove
apt-get -y clean