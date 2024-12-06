#!/bin/sh


docker buildx build \
       --platform linux/arm64 \
       -t "leighpauls/manytris-manager:dev" \
       --progress=plain \
       --build-arg BUILD_PROFILE=dev \
       --build-arg TARGET_DIR=debug \
       -f Dockerfile.manager \
       .

