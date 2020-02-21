#!/usr/bin/env bash

if [ "$EUID" -ne 0 ]; then
    echo "Please run this setup script, setup_server.sh as root."
    exit
fi

# Make a directory for the rust-summercash repo
sudo mkdir -p ~/rust/src/github.com/SummerCash && cd ~/rust/src/github.com/SummerCash

# Make sure git is installed
if hash apt 2>/dev/null; then
    echo "== DEPS == installing git via apt..."
    sudo apt-get install -y git
    echo "== SUCCESS == done installing git!"

    echo -e "\n== DEPS == installing cc linker utils..."
    sudo apt-get install -y build-essential
    echo "== SUCCESS == done installing cc linker utils!"

    echo -e "\n== DEPS == installing libssl..."
    sudo apt-get install -y libssl-dev
    echo "== SUCCESS == done installing openssl!"

    echo -e "\n== DEPS == installing pkgconfig..."
    sudo apt-get install -y pkg-config
    echo "== SUCCESS == done installing pkgcfg!"
elif hash brew 2>/dev/null; then 
    echo "== DEPS == installing git via homebrew..."
    sudo brew install git
else
    echo "unable to install git :("
    exit
fi

# Download the rust-summercash source code
echo -e "\n== SRC == cloning rust-summercash into $HOME/rust/src/github.com/SummerCash"
git clone https://github.com/SummerCash/rust-summercash.git

# Make sure rust is installed
if ! [ hash cargo 2>/dev/null ]; then
    echo -e "\n== DEPS == installing cargo..."

    # Install rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup-init.sh
    chmod +x rustup-init.sh && ./rustup-init.sh -y
fi

# Add cargo to the PATH
source $HOME/.cargo/env

# Compile the SummerCash soure code
echo -e "\n== SRC == compiling rust-summercash..."
cd rust-summercash && cargo build --release

echo -e "\n== SERVICE == creating a systemd service file in /etc/systemd/system for SMCD wherein the executable for SMCD: $HOME/rust/src/github.com/SummerCash/rust-summercash/target/release/smcd will be used..."

# Create a systemd service file for the SummerCash daemon
sudo touch /etc/systemd/system/smcd.service

# Put a service definition into this new file
sudo echo "[Unit]
Description=SummerCash daemon

[Service]
ExecStart=$HOME/rust/src/github.com/SummerCash/rust-summercash/target/release/smcd -p 2048

[Install]
WantedBy=multi-user.target" > /etc/systemd/system/smcd.service

echo -e "\n== SERVICE == Starting smcd..."

# Start the SummerCash daemon
sudo systemctl daemon-reload && sudo systemctl start smcd
