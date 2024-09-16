#!/bin/bash

response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"email":"admin@localhost","password":"administrador.semarh","scopes":[]}' http://localhost:8000/api/v1/auth)
token=$(echo $response | jq -r '.token')
if [ -z "$token" ]
then
  echo "Token not found"
  exit 1
fi

export API_TOKEN=$token

wrk -c 25 -t 25 -d 10s -H "Authorization: Bearer $API_TOKEN" --latency --timeout 2s http://127.0.0.1:8000/api/v1/users
