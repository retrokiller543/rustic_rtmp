# Use the latest official Rust image as the base for compilation
FROM rust as builder

# Install nightly Rust and set it as default
RUN rustup install nightly && rustup default nightly  

# Create the app directory
WORKDIR /app

# Copy the entire project into the image (ensure you have a .dockerignore to exclude target/)
COPY . .

# Compile the Rust application in the Docker environment
RUN cargo build --release

# Use a newer base image for the runtime, ensure it has the required glibc version
FROM debian:bullseye-slim

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/rustic_rtmp /app/

# Set the working directory
WORKDIR /app

# Command to run the application when the container starts
CMD ["./rustic_rtmp"]
