#!/usr/bin/env bash

if [ "$EUID" -ne 0 ]; then
    echo "Please run this setup script, setup_server.sh as root."
    exit
fi

# Make a directory for the rust-summercash repo
mkdir -p ~/rust/src/github.com/SummerCash && cd ~/rust/src/github.com/SummerCash

# Make sure git is installed
if hash apt 2>/dev/null; then
    echo "installing git via apt..."
    sudo apt-get install -y git
elif hash brew 2>/dev/null; then 
    echo "installing git via homebrew..."
    sudo brew install git
else
    echo "unable to install git :("
    exit
fi

# Download the rust-summercash source code
git clone https://github.com/SummerCash/rust-summercash.git

# Make sure rust is installed
if ! [ hash apt 2>/dev/null ]; then
    # Install rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup-init.sh
    chmod +x rustup-init.sh && ./rustup-init.sh -y
fi

# Compile the SummerCash soure code
cargo build --release

# Create a systemd service file for the SummerCash daemon
sudo touch /etc/systemd/system/smcd.service

# Put a service definition into this new file
sudo echo "[Unit]
Description=SummerCash daemon

[Service]
ExecStart=$HOME/src/github.com/SummerCash/rust-summercash/target/release/smcd -p 2048

[Install]
WantedBy=multi-user.target" > /etc/systemd/system/smcd.service

echo "Starting smcd..."

# Start the SummerCash daemon
sudo systemctl daemon-reload && sudo systemctl start smcd
