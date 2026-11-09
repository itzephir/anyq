FROM nginxinc/nginx-unprivileged:1.28-alpine
COPY website/ingress.conf /etc/nginx/conf.d/default.conf
EXPOSE 8080
