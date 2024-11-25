#!/bin/sh


docker buildx build --platform linux/arm64 -t leighpauls/manytris:dev --progress=plain .
