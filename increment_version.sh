#!/bin/bash

tag=$(git describe --tags --abbrev=0)

major=$(echo $tag | cut -d. -f1 | cut -c2-)
minor=$(echo $tag | cut -d. -f2)
patch=$(echo $tag | cut -d. -f3)

messages=$(git log $tag..HEAD --pretty=format:%s)

if echo "$messages" | grep -q "^feat:"; then
  # Increment the major number
  major=$((major + 1))
  minor=0
  patch=0
elif echo "$messages" | grep -q "^fix:"; then
  # Increment the minor number
  minor=$((minor + 1))
  patch=0
elif echo "$messages" | grep -qE "^(chore|refactor):"; then
  # Increment the patch number
  patch=$((patch + 1))
fi

echo "v$major.$minor.$patch"