echo "Preparing env file"

set -e -u

PRIVATE_KEY=$(cat ./keys/jwtRS256.key)
PUBLIC_KEY=$(cat ./keys/jwtRS256.key.pub)

cat ./.env.template > ./.env

echo "JWT_PUBLIC_KEY=\"${PUBLIC_KEY}\"" >> ./.env
echo "JWT_PRIVATE_KEY=\"${PRIVATE_KEY}\"" >> ./.env
