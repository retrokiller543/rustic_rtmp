#!/bin/bash

# Source the .env file to load the environment variables
source .env

# Check if the .env file was sourced successfully
if [[ -z "$DEPLOY_USERNAME" || -z "$DEPLOY_ADDRESS" ]]; then
  echo "Error: Missing environment variables. Please check your .env file."
  exit 1
fi

# Array of target platforms
declare -a targets=("armv7-unknown-linux-gnueabihf")

for target in "${targets[@]}"; do
  echo "Compiling for $target..."
  cargo build --release --target="$target"
  echo "Copying binary to $DEPLOY_ADDRESS..."
  ssh "$DEPLOY_USERNAME"@"$DEPLOY_ADDRESS" "mkdir -p /home/$DEPLOY_USERNAME/rustic_rtmp/$target"
  scp -r target/"$target"/release "$DEPLOY_USERNAME"@"$DEPLOY_ADDRESS":/home/"$DEPLOY_USERNAME"/rustic_rtmp/"$target"
  ssh "$DEPLOY_USERNAME"@"$DEPLOY_ADDRESS" "/home/"$DEPLOY_USERNAME"/rustic_rtmp/"$target"/rustic_rtmp"
done
