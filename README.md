A database to store rides on public transports and tag them with
metadata.

# Installation

1. Fetch files.
   ```shell
   curl -o docker-compose.yml https://github.com/pl33/public-transport-expense-tracker-backend/raw/refs/heads/main/docker-compose.yml
   curl -o .env https://raw.githubusercontent.com/pl33/public-transport-expense-tracker-backend/refs/heads/main/example.env
   curl -o prepare.sh https://raw.githubusercontent.com/pl33/public-transport-expense-tracker-backend/refs/heads/main/prepare.sh
   curl -o make_jwt.sh https://raw.githubusercontent.com/pl33/public-transport-expense-tracker-backend/refs/heads/main/make_jwt.sh
   chmod +x prepare.sh make_jwt.sh
   ```
2. Modify `.env` File. It is important to set the `SECRET_KEY` field.
3. Optionally, adapt `docker-compose.yml`.
4. Run
   ```shell
   ./prepare.sh
   ```
5. Execute the container
   ```shell
   docker-compose up -d
   ```
   
# Maintenance

## Create JWTs

```shell
./make_jwt.sh
```
