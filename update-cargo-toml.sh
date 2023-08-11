#!/bin/bash

version=$(./increment-version.sh)

sed -i "s/^version = .*/version = "$version"/" Cargo.toml