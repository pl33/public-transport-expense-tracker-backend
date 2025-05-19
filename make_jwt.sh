#!/bin/bash

source .env
docker run --rm -ti -v "./data/keys/:/data/keys:ro" ghcr.io/pl33/public-transport-expense-tracker-backend:latest token --key-dir /data/keys create-token --issuer $JWT_ISSUER --audience $BASE_URI -e $(date --utc --date="+364 days" +%FT%XZ) --claims-json '{"ptet:write":true}' $1