
{ echo "SQLX Migration"; } 2> /dev/null

set -e

sqlx migrate run
