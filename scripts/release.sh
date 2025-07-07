#!/bin/bash

# Simple semantic versioning helper script
# Usage: ./scripts/release.sh [major|minor|patch]

set -e

# Default to patch if no argument provided
VERSION_TYPE=${1:-patch}

# Get the current version from git tags
CURRENT_VERSION=$(git tag -l "v*.*.*" | sort -V | tail -1)

if [ -z "$CURRENT_VERSION" ]; then
    echo "No existing version tags found. Starting with v0.1.0"
    NEW_VERSION="v0.1.0"
else
    echo "Current version: $CURRENT_VERSION"
    
    # Remove the 'v' prefix for processing
    VERSION_NUMBER=${CURRENT_VERSION#v}
    
    # Split version into components
    IFS='.' read -ra VERSION_PARTS <<< "$VERSION_NUMBER"
    MAJOR=${VERSION_PARTS[0]}
    MINOR=${VERSION_PARTS[1]}
    PATCH=${VERSION_PARTS[2]}
    
    # Increment based on type
    case $VERSION_TYPE in
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        patch)
            PATCH=$((PATCH + 1))
            ;;
        *)
            echo "Error: Invalid version type '$VERSION_TYPE'. Use 'major', 'minor', or 'patch'"
            exit 1
            ;;
    esac
    
    NEW_VERSION="v$MAJOR.$MINOR.$PATCH"
fi

echo "New version: $NEW_VERSION"

# Confirm before tagging
read -p "Create and push tag $NEW_VERSION? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Create and push the tag
    git tag -a "$NEW_VERSION" -m "Release $NEW_VERSION"
    git push origin "$NEW_VERSION"
    
    echo "Tag $NEW_VERSION created and pushed successfully!"
    echo "Container build workflow will be triggered automatically."
    echo "Check the Actions tab on GitHub for build progress."
else
    echo "Tag creation cancelled."
fi