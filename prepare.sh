#!/bin/sh
export DOCKER_BUILDKIT=1 
export SKIP_SWIFT_BUILD=1

cd src-tauri/rs-azookey-binding
docker build --target export --output type=local,dest=./ .
