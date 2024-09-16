#!/bin/bash

response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"email":"admin@localhost","password":"administrador.semarh","scopes":[]}' http://localhost:8000/api/v1/auth)
token=$(echo $response | jq -r '.token')
if [ -z "$token" ]
then
  echo "Token not found"
  exit 1
fi

export API_TOKEN=$token

drill --benchmark drill_benchmark.yml --stats --quiet
