#!/bin/bash

# Wait for Keycloak to be ready
echo "Waiting for Keycloak to be ready..."
until curl -s http://localhost:8080/auth/realms/master > /dev/null; do
    sleep 1
done

# Get admin token
echo "Getting admin token..."
TOKEN=$(curl -s -X POST http://localhost:8080/auth/realms/master/protocol/openid-connect/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=admin" \
  -d "password=admin" \
  -d "grant_type=password" \
  -d "client_id=admin-cli" | jq -r '.access_token')

# Import realm
echo "Importing realm configuration..."
curl -s -X POST http://localhost:8080/auth/admin/realms \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @../config/realm-export.json

echo "Realm configuration imported successfully!" 