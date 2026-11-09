#!/usr/bin/env bash
set -euo pipefail
export ANYQ_SITE_CONTAINER=anyq-site-ci
export ANYQ_SITE_NETWORK=anyq-site-ci-network
wrapper_dir=$(mktemp -d)
cleanup() {
    docker rm -f anyq-site-ci anyq-site-ci-candidate anyq-site-ci-previous >/dev/null 2>&1 || true
    docker network rm "$ANYQ_SITE_NETWORK" >/dev/null 2>&1 || true
    rm -rf "$wrapper_dir"
}
trap cleanup EXIT
docker network create "$ANYQ_SITE_NETWORK" >/dev/null
bash tooling/deploy_website.sh anyq-site:test
original=$(docker inspect --format '{{.Id}}' anyq-site-ci)
printf 'FROM anyq-site:test\nUSER root\nRUN rm /usr/share/nginx/html/index.html\nUSER 101\n' | docker build -q -t anyq-site:broken - >/dev/null
if bash tooling/deploy_website.sh anyq-site:broken; then
    echo 'Broken website was accepted' >&2
    exit 1
fi
test "$(docker inspect --format '{{.Id}}' anyq-site-ci)" = "$original"
# Simulate a crash after candidate validation and the production name swap.
export ANYQ_REAL_DOCKER
ANYQ_REAL_DOCKER=$(command -v docker)
cat > "$wrapper_dir/docker" <<'WRAPPER'
#!/usr/bin/env bash
set -euo pipefail
args=("$@")
crash=false
if [[ ${args[0]} == run ]]; then
    for ((index=0; index<${#args[@]}-1; index++)); do
        if [[ ${args[index]} == --name && ${args[index+1]} == "$ANYQ_SITE_CONTAINER" ]]; then
            crash=true
        fi
    done
fi
"$ANYQ_REAL_DOCKER" "$@"
if [[ $crash == true ]]; then
    "$ANYQ_REAL_DOCKER" stop "$ANYQ_SITE_CONTAINER" >/dev/null
fi
WRAPPER
chmod +x "$wrapper_dir/docker"
if PATH="$wrapper_dir:$PATH" bash tooling/deploy_website.sh anyq-site:test; then
    echo 'Crashed production container was accepted' >&2
    exit 1
fi
test "$(docker inspect --format '{{.Id}}' anyq-site-ci)" = "$original"
docker exec anyq-site-ci wget -q -O - http://127.0.0.1:8080/healthz | grep -qx anyq
docker exec anyq-site-ci sh -c 'test "$(wget -q -O - http://127.0.0.1:8080/install.sh | head -1)" = "#!/bin/sh"'
bash tooling/deploy_website.sh anyq-site:test
echo 'Website container update, failed-candidate preservation and activation rollback passed'
