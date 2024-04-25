echo "RSA Key Pair setup"

set -e

OUT_DIR=keys

mkdir -p ${OUT_DIR}

rm -f ./${OUT_DIR}/jwtRS256.key ./${OUT_DIR}/jwtRS256.key.pub

ssh-keygen -t rsa -b 4096 -m PKCS8 -f ./${OUT_DIR}/jwtRS256.key -N ''

# Don't add passphrase
openssl rsa -in ./${OUT_DIR}/jwtRS256.key -pubout -outform PEM -out ./${OUT_DIR}/jwtRS256.key.pub
