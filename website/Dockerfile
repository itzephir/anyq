FROM python:3.12-alpine AS build
WORKDIR /src
COPY website/ website/
COPY install/ install/
COPY tooling/build_website.py tooling/build_website.py
RUN python tooling/build_website.py

FROM nginxinc/nginx-unprivileged:1.28-alpine
COPY website/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /src/website/dist/ /srv/site/current/
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=3s --start-period=5s CMD wget -q -O /dev/null http://127.0.0.1:8080/ || exit 1
