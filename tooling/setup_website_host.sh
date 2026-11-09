#!/usr/bin/env bash
# One-time operator setup; project CI receives only stand-scoped forced SSH commands.
set -euo pipefail
[[ $EUID == 0 ]] || { echo 'Run as the VM operator' >&2; exit 1; }
revision=${1:?Usage: setup_website_host.sh COMMIT_SHA}
[[ $revision =~ ^[a-f0-9]{40}$ ]]
label=io.anyq.website
docker network inspect itzephir-projects-prod >/dev/null
if [[ -d /srv/anyq && ! -f /srv/anyq/.website-managed ]] && [[ -n $(find /srv/anyq -mindepth 1 -maxdepth 1 -print -quit) ]]; then
    echo 'Refusing an unmanaged /srv/anyq directory' >&2
    exit 1
fi
for name in anyq-gateway anyq-site-dev anyq-site-prod; do
    if docker container inspect "$name" >/dev/null 2>&1; then
        test "$(docker inspect --format '{{ index .Config.Labels "io.anyq.website" }}' "$name")" = true || {
            echo "Refusing an unmanaged container: $name" >&2
            exit 1
        }
    fi
done
if docker network inspect anyq-web >/dev/null 2>&1; then
    test "$(docker network inspect --format '{{ index .Labels "io.anyq.website" }}' anyq-web)" = true
else
    docker network create --label "$label=true" anyq-web >/dev/null
fi
install -d -m 755 /srv/anyq/dev/releases /srv/anyq/prod/releases /usr/local/libexec/anyq
printf 'anyq website hosting v1\n' > /srv/anyq/.website-managed
install -m 755 tooling/receive_website.py /usr/local/libexec/anyq/receive-website.py
docker build -q -f website/Dockerfile -t "anyq-site-host:$revision" . >/dev/null
docker build -q -f website/ingress.Dockerfile -t "anyq-ingress:$revision" . >/dev/null
for stand in dev prod; do
    if ! docker container inspect "anyq-site-$stand" >/dev/null 2>&1; then
        docker run -d --name "anyq-site-$stand" --label "$label=true" --restart unless-stopped \
            --network anyq-web --read-only --tmpfs /tmp:rw,nosuid,size=16m \
            --memory 64m --cpus .25 --pids-limit 64 --cap-drop ALL --security-opt no-new-privileges \
            --mount "type=bind,source=/srv/anyq/$stand,target=/srv/site,readonly" \
            "anyq-site-host:$revision" >/dev/null
    fi
done
if ! docker container inspect anyq-gateway >/dev/null 2>&1; then
    docker run -d --name anyq-gateway --label "$label=true" --restart unless-stopped \
        --network anyq-web --read-only --tmpfs /tmp:rw,nosuid,size=16m \
        --memory 64m --cpus .25 --pids-limit 64 --cap-drop ALL --security-opt no-new-privileges \
        "anyq-ingress:$revision" >/dev/null
fi
if ! docker inspect --format '{{json .NetworkSettings.Networks}}' anyq-gateway | grep -q '"itzephir-projects-prod"'; then
    docker network connect itzephir-projects-prod anyq-gateway
fi
echo 'anyq hosting prepared; CI can publish dev/prod through separate restricted keys'
