# Stage 1: Build the Rust application
# Use the official Rust image
FROM rust:1.70 AS builder

# Create a new empty shell project to cache dependencies
WORKDIR /usr/src/app
RUN USER=root cargo new --bin my_project
WORKDIR /usr/src/app/my_project

# Copy over your manifests
COPY Cargo.toml Cargo.lock ./

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Now, copy your actual source code
COPY src ./src

# Build the application for release, which will be much faster due to the cached dependencies
RUN rm ./target/release/deps/my_project*
RUN cargo build --release

# Stage 2: Create the final, minimal image
# Use a slim Debian image for a smaller footprint
FROM debian:bullseye-slim

# Set a non-root user for security
RUN groupadd -r appuser && useradd -r -g appuser appuser
USER appuser

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/my_project/target/release/my_project /usr/local/bin/my_project

# Set the command to run your application
# Ensure your Rust app listens on 0.0.0.0 to be accessible from outside the container
CMD ["/usr/local/bin/my_project"]
