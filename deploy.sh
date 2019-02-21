#!/bin/sh
make
sudo systemctl stop todo.service
sudo cp backend/target/release/backend /usr/local/bin/todo
sudo systemctl start todo.service
