#!/bin/bash

# Use Eric Kidd's magical docker image
# https://github.com/emk/rust-musl-builder

(
  cd backend
  docker run -p 192.168.1.37:5000:8000 --rm -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder ./.container-script.sh
)
