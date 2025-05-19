#!/bin/bash

docker run --rm -ti -v "./data/keys/:/data/keys" ghcr.io/pl33/public-transport-expense-tracker-backend:latest token --key-dir /data/keys create-key