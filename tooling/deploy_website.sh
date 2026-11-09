#!/usr/bin/env bash
# Run on the existing Docker host with a reviewed, locally available image.
set -euo pipefail
image=${1:?Usage: deploy_website.sh IMAGE}
name=${ANYQ_SITE_CONTAINER:-anyq-site}
network=${ANYQ_SITE_NETWORK:-itzephir-projects-prod}
candidate=$name-candidate
previous=$name-previous
label=com.itzephir.anyq.site

exec 9>"/tmp/$name-deploy.lock"
flock -x 9
docker network inspect "$network" >/dev/null
docker image inspect "$image" >/dev/null
for container in "$name" "$candidate" "$previous"; do
    if docker container inspect "$container" >/dev/null 2>&1; then
        test "$(docker inspect --format '{{ index .Config.Labels "com.itzephir.anyq.site" }}' "$container")" = true || {
            echo "Refusing to replace an unmanaged container: $container" >&2
            exit 1
        }
    fi
done

start() {
    docker run -d --name "$1" --label "$label=true" --restart unless-stopped \
        --network "$network" --memory 64m --cpus .25 --pids-limit 64 \
        --security-opt no-new-privileges --cap-drop ALL "$image" >/dev/null
}
healthy() {
    for ((attempt=0; attempt<20; attempt++)); do
        if docker exec "$1" sh -c 'wget -q -O /dev/null http://127.0.0.1:8080/ && wget -q -O /dev/null http://127.0.0.1:8080/install.sh && wget -q -O /dev/null http://127.0.0.1:8080/install.ps1' 2>/dev/null; then
            return 0
        fi
        sleep 1
    done
    return 1
}
activated=false
had_previous=false
old_renamed=false
cleanup() {
    result=$?
    trap - EXIT
    docker rm -f "$candidate" >/dev/null 2>&1 || true
    if [[ $result != 0 && $activated == true ]]; then
        if [[ $old_renamed == true ]]; then
            docker rm -f "$name" >/dev/null 2>&1 || true
            docker rename "$previous" "$name"
            docker start "$name" >/dev/null
        elif [[ $had_previous == true ]]; then
            docker start "$name" >/dev/null
        else
            docker rm -f "$name" >/dev/null 2>&1 || true
        fi
    fi
    exit "$result"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

docker rm -f "$candidate" >/dev/null 2>&1 || true
start "$candidate"
healthy "$candidate"
docker rm -f "$candidate" >/dev/null
docker rm -f "$previous" >/dev/null 2>&1 || true
if docker container inspect "$name" >/dev/null 2>&1; then
    had_previous=true
    activated=true
    docker stop "$name" >/dev/null
    docker rename "$name" "$previous"
    old_renamed=true
fi
activated=true
start "$name"
healthy "$name"
docker rm -f "$previous" >/dev/null 2>&1 || true
echo "anyq website is healthy: $image"
