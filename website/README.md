# anyq.itzephir.com

Исходники лендинга, установщиков, контейнера и доставки принадлежат anyq.
Дизайн описан в [DESIGN.md](DESIGN.md).

## Просмотр и OpenDesign

```sh
python3 tooling/build_website.py
python3 -m http.server 4192 --directory website/dist --bind 127.0.0.1
```

Открыть http://127.0.0.1:4192/?theme=dark и ?theme=light. Проверять 320/390/1440 px,
фокус клавиатуры, копирование, обе системы установки и содержимое без JS.
В OpenDesign отдельный проект anyq связан с user:itzephir. Переносить только
website/dist, без checkout и credentials. Пути удалённого CLI относятся к серверу.

## Доставка

```sh
docker build -f website/Dockerfile -t anyq-site:review .
```

CI проверяет HTTP, скрипты и сохранение работающего контейнера при ошибке candidate.
Main публикует ghcr.io/itzephir/anyq-site с тегом commit SHA. Пакет GHCR должен
быть доступен оператору для pull; видимость package настраивается в GitHub Packages.
Workflow не меняет доступы и не получает SSH ключей.

На существующем Docker-хосте после review/merge:

```sh
docker pull ghcr.io/itzephir/anyq-site:COMMIT_SHA
bash tooling/deploy_website.sh ghcr.io/itzephir/anyq-site:COMMIT_SHA
```

Сценарий проверяет candidate и возвращает прежний контейнер при ошибке активации.
Сеть — itzephir-projects-prod; порт 8080 доступен только внутри Docker. Host port,
новые сертификаты и DNS не нужны. В панели itzephir.com/dev: slug anyq, Docker
target anyq-site:8080. До проверки https://anyq.itzephir.com/ готовность planning;
затем development. Откат — тот же сценарий с прежним digest/tag образа.

До первого полного GitHub Release страница показывает предупреждение,
а установщик завершается ошибкой без изменения старой установки. После выпуска
удалить `.release-note` из index.html отдельным PR.
