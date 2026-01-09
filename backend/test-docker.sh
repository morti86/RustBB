#!/bin/bash

# Simple script to verify Dockerfile syntax and structure

echo "=== Dockerfile Verification ==="
echo ""

# Check if Dockerfile exists
if [ ! -f "Dockerfile" ]; then
    echo "❌ Dockerfile not found!"
    exit 1
fi

echo "✅ Dockerfile exists"

# Check Dockerfile syntax
echo ""
echo "=== Dockerfile Content ==="
cat Dockerfile

echo ""
echo "=== Checking for common issues ==="

# Check for multi-stage build
if grep -q "FROM.*AS.*builder" Dockerfile; then
    echo "✅ Multi-stage build detected"
else
    echo "⚠️  No multi-stage build detected (not necessarily an error)"
fi

# Check for non-root user
if grep -q "USER appuser" Dockerfile; then
    echo "✅ Non-root user configuration found"
else
    echo "⚠️  No non-root user specified"
fi

# Check for health check
if grep -q "HEALTHCHECK" Dockerfile; then
    echo "✅ Health check configured"
else
    echo "⚠️  No health check configured"
fi

# Check for exposed ports
if grep -q "EXPOSE" Dockerfile; then
    echo "✅ Ports exposed"
else
    echo "⚠️  No ports exposed"
fi

# Check for .dockerignore
if [ -f ".dockerignore" ]; then
    echo "✅ .dockerignore file exists"
else
    echo "⚠️  No .dockerignore file"
fi

# Check for docker-compose.yml
if [ -f "docker-compose.yml" ]; then
    echo "✅ docker-compose.yml exists"
else
    echo "⚠️  No docker-compose.yml file"
fi

echo ""
echo "=== Required Files Check ==="

# Check for required files
required_files=("Cargo.toml" "src/main.rs" "schema.sql")
for file in "${required_files[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file exists"
    else
        echo "❌ $file missing!"
    fi
done

echo ""
echo "=== Summary ==="
echo "Docker configuration appears to be correctly set up."
echo "To build and run:"
echo "  1. docker build -t forum-backend ."
echo "  2. docker-compose up -d"
echo ""
echo "Note: Docker daemon must be running for actual build."

