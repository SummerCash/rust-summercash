# Use the 1.40 build of rustc
FROM rust:1.40

# Copy the entirety of the SummerCash source code
COPY ../../../ ./

# Build and optimize the SummerCash source
RUN cargo build --release

# Install the smcd and smcli commands
RUN cargo install --path .

ENTRYPOINT smcd