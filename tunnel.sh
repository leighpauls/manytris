#!/bin/bash

socat TCP-LISTEN:9988,reuseaddr,fork TCP:localhost:9989
