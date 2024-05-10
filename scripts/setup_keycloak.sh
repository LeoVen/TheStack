echo "Keycloak setup"

set -e -u

. ./.env

REALM_NAME=${KC_SETUP_REALM}
USER_NAME=${KC_SETUP_USER_NAME}
USER_PASSWORD=${KC_SETUP_USER_PASSWORD}

{ echo "Waiting for Keycloak Server"; } 2> /dev/null

curl --retry 20 -f --retry-all-errors --retry-delay 2 -s -o /dev/null "http://localhost:8080/health"

sleep 2 # reduce flakyness

{ echo "Getting admin credentials"; } 2> /dev/null

ACCESS_TOKEN=$(curl -s --request POST \
  --url http://localhost:8080/realms/master/protocol/openid-connect/token \
  --header 'content-type: application/x-www-form-urlencoded' \
  --data username=admin \
  --data password=admin \
  --data client_id=admin-cli \
  --data grant_type=password \
  | jq -r ".access_token")

{ echo "Creating Realm \"${REALM_NAME}\""; } 2> /dev/null

curl -s --request POST \
  --url http://localhost:8080/admin/realms \
  --header "authorization: bearer ${ACCESS_TOKEN}" \
  --header 'content-type: application/json' \
  --data "{\"realm\": \"${REALM_NAME}\", \"enabled\": \"true\"}"

{ echo "Creating User \"${USER_NAME}\""; } 2> /dev/null

curl -s --request POST \
  --url http://localhost:8080/admin/realms/${REALM_NAME}/users \
  --header "authorization: bearer ${ACCESS_TOKEN}" \
  --header 'content-type: application/json' \
  --header 'user-agent: vscode-restclient' \
  --data "{\"firstName\": \"S2S User\", \"lastName\": \"Auto Generated\",\"username\": \"${USER_NAME}\",\"email\": \"thestack@mail.com\", \"emailVerified\": \"true\",\"enabled\": \"true\",\"credentials\": [{\"type\": \"password\",\"value\": \"${USER_PASSWORD}\",\"temporary\": \"false\"}]}"

{ echo "Keycloak setup done"; } 2> /dev/null
