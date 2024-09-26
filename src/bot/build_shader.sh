#!/bin/bash

set -e

xcrun --sdk macosx metal -o bot_shader.air -c bot_shader.metal
xcrun --sdk macosx metallib -o bot_shader.metallib bot_shader.air

