# anyq website and stands

Лендинг, установщики и доставка принадлежат репозиторию anyq. Общий шлюз itzephir
выбирает проект anyq; контейнер anyq-gateway выбирает стенд по публичному Host:

| Адрес | Стенд | Публикация |
| --- | --- | --- |
| https://dev.anyq.itzephir.com/ | dev | После успешного CI PR из этого репозитория |
| https://anyq.itzephir.com/ | prod | После успешного CI push в main |

Fork/Dependabot PR не получают ключей. Один dev-стенд общий для PR: последняя
завершённая выкладка обновляет его. Выкладки одного стенда сериализованы;
текущая версия доступна в /revision.txt. Main не обновляет dev автоматически.
До первого merge production-контейнер подготовлен, но без опубликованного сайта:
его запросы возвращают 503. Разные ключи dev/prod не могут выбрать другой стенд.

## Локальная проверка и OpenDesign

```sh
python3 tooling/build_website.py
python3 -m http.server 4192 --directory website/dist --bind 127.0.0.1
```

Проверять ?theme=dark/light, 320/390/1440 px, клавиатуру и содержимое без JS.
Дизайн описан в [DESIGN.md](DESIGN.md). В OpenDesign проект anyq связан с
user:itzephir; переносить только website/dist. Пути удалённого CLI — серверные.

```sh
python3 -m unittest discover -s tooling -p receiver_website_test.py
docker build -f website/Dockerfile -t anyq-site:test .
python3 tooling/hosting_website_test.py
python3 tooling/package_website.py
```

Последняя команда создаёт website/site.zip только из публичных файлов. CI
проверяет образ, ZIP boundary, откат и реальный ingress, затем сохраняет ZIP
как anyq-website. Тот же проверенный артефакт отправляется на выбранный стенд.

## Однократная операторская подготовка

На существующей ВМ из проверенного checkout anyq:

```sh
bash tooling/setup_website_host.sh COMMIT_SHA
```

Сценарий запускают с операторскими правами. Он готовит anyq-web, anyq-gateway
и anyq-site-dev/prod. Ingress подключён к itzephir-projects-prod, backend-контейнеры
находятся только в приватной сети anyq-web. Host ports не открываются.
Публичные файлы стендов хранятся в /srv/anyq/dev и /srv/anyq/prod; mounts read-only.
Receiver устанавливается в root-owned /usr/local/libexec/anyq/receive-website.py.
Повторная подготовка сохраняет существующие управляемые контейнеры и версии;
изменение их образов/конфигурации требует отдельной операторской выкладки.

В production locator настроить slug anyq, Docker target anyq-gateway:8080,
approved/development. Владелец itzephir включает namespace anyq и DNS/TLS
для anyq.itzephir.com и *.anyq.itzephir.com существующим инфраструктурным процессом.
Проектная регистрация не выполняет сборку и не создаёт стенды.

## Доступ GitHub Actions

GitHub environments development и production содержат отдельные:
ANYQ_DEPLOY_HOST, ANYQ_DEPLOY_KEY и ANYQ_DEPLOY_KNOWN_HOSTS. Production environment
допускает только branch main. Приватные адреса и значения секретов не публикуются.

У каждого ключа в authorized_keys включены restrict и отдельный forced command:

```text
/usr/bin/python3 -I /usr/local/libexec/anyq/receive-website.py dev
/usr/bin/python3 -I /usr/local/libexec/anyq/receive-website.py prod
```

Ключи предназначены только для этого проекта; существующий операторский SSH-ключ
не копируется в CI. Receiver принимает единственную команду deploy COMMIT_SHA и ZIP
через stdin. Произвольный shell, Docker-образ, Dockerfile, скрипт установки хоста
или выбор другого стенда не принимаются. Пути, типы и размеры файлов проверяются
до записи. Ревизия не может быть повторно опубликована с другим содержимым.

## Обновление и откат

Receiver сохраняет полный release, атомарно переключает current и проверяет HTTP
revision в соответствующем nginx. Ошибка возвращает прежний current. После этого
CI отдельно проверяет публичный HTTPS и совпадение revision с проверенным commit.
Оператор может вернуть любой сохранённый release через повторную публикацию его
ZIP с прежним SHA. Файлы CLI, настройки и очереди пользователей не затрагиваются.

Первый GitHub Release бинарников выпускается отдельно. До него установщик
сообщает об отсутствии версии и сохраняет прежнюю установку. Пользователь убрал
служебные предупреждения из самого лендинга; статус выпуска остаётся в документации.
