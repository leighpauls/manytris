#!/bin/sh

if [[ $# -ne 1 ]] ; then
    echo "usage: $0 <tag>"
    exit 1
fi

docker buildx build \
       --platform linux/amd64,linux/arm64 \
       -t "leighpauls/manytris:$1" \
       --progress=plain \
       --build-arg BUILD_PROFILE=release \
       --build-arg TARGET_DIR=release \
       -f Dockerfile.game \
       . \
       --push
