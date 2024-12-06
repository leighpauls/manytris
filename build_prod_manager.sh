#!/bin/sh

if [[ $# -ne 1 ]] ; then
    echo "usage: $0 <tag>"
    exit 1
fi


docker buildx build \
       --platform linux/arm64,linux/amd64 \
       -t "leighpauls/manytris-manager:$1" \
       --progress=plain \
       --build-arg BUILD_PROFILE=dev \
       --build-arg TARGET_DIR=debug \
       -f Dockerfile.manager \
       . \
       --push

