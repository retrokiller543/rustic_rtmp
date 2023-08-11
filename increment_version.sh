#!/bin/bash

tag=$(git describe --tags --abbrev=0)

major=$(echo $tag | cut -d. -f1 | cut -c2-)
minor=$(echo $tag | cut -d. -f2)
patch=$(echo $tag | cut -d. -f3)

messages=$(git log $tag..HEAD --pretty=format:%s)

major_count=$(echo "$messages" | grep -cE "^(feat|major):")
minor_count=$(echo "$messages" | grep -cE "^(fix|minor):")
patch_count=$(echo "$messages" | grep -cE "^(chore|refactor|patch):")

if [[ $major_count -ge $minor_count ]] && [[ $major_count -ge $patch_count ]]; then
  # Increment the major number
  major=$((major + 1))
  minor=0
  patch=0
elif [[ $minor_count -ge $major_count ]] && [[ $minor_count -ge $patch_count ]]; then
  # Increment the minor number
  minor=$((minor + 1))
  patch=0
else
  # Increment the patch number
  patch=$((patch + 1))
fi

version="$major.$minor.$patch"

# Update the version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"$version\"/" Cargo.toml

echo $version
