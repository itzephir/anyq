# Design

## Context

См. proposal.md. Runtime уже нативный и SQLite включён в бинарник. Портфолио имеет project registry и общий Docker gateway. OpenDesign доступен через локальный CLI с существующим SSH-доступом.

## Goals / Non-Goals

**Goals:** исходники страницы, установщиков и доставки принадлежат anyq; проверка архивов и платформ выполняется до публикации; макет использует ту же страницу.

**Non-Goals:** изменение очереди, добавление GUI для задач, перенос portfolio shell или автоматическое слияние PR.

## Decisions

- Статический HTML/CSS/JS с локальными IBM Plex и семантическими токенами itzephir. Страница объясняет atomic admission и explicit complete. Небольшой JS сохраняет тему и копирует команды; основное содержание работает без JS.
- Shell для Unix и PowerShell 5.1+ для Windows. Архивы имеют фиксированные имена по Rust target, SHA256SUMS скачивается из той же версии. latest определяется редиректом GitHub; pin версии доступен аргументом. Unix устанавливает в ~/.local/bin (или --bin-dir), Windows — LOCALAPPDATA/anyq/bin с обновлением пользовательского PATH.
- Linux musl для переносимости, нативные macOS targets и MSVC Windows x64. Сначала CI тестирует установку локальных артефактов через HTTP fixtures, затем релиз собирает тот же формат. Release публикуется только после получения всех артефактов и проверки соответствия тега версии Cargo.
- Проектный nginx ingress в существующей сети gateway выбирает dev/prod по Host. Два статических контейнера читают собственный current из read-only mounts в приватной сети anyq. CI передаёт ZIP проверенного сайта по SSH; ограниченный receiver принимает только публичные файлы, атомарно переключает current и проверяет HTTP. Ошибка возвращает прежнюю ссылку. SSH-ключи dev/prod разделены, forced command не даёт произвольного shell. Production environment ограничен main. Адреса и credentials хранятся только в секретах GitHub/операторской конфигурации.
- В OpenDesign создаётся отдельный проект anyq с designSystemId user:itzephir. Загружается allowlist собранного сайта, а не checkout. Проверяется совпадение файлов.

## Risks / Trade-offs

- Первый релиз отсутствует → установщик сообщает об ошибке GitHub; документация явно отделяет готовый код от опубликованной версии.
- Checksum из того же HTTPS источника не является независимой подписью → SHA-256 защищает целостность загрузки; GitHub Release остаётся доверенным источником.
- macOS бинарники без notarization → CLI распространяется архивом через HTTPS, без обещания Apple signing.
- Linux ARM сборка → отдельный нативный ARM runner и musl toolchain.

## Migration Plan

PR проверяет обязательный CI, installer fixtures и контейнеры; после успеха публикует ZIP на dev.anyq.itzephir.com. Main после merge публикует production. Оператор один раз устанавливает проектные контейнеры и receiver и регистрирует anyq-gateway:8080 в существующем locator; DNS/TLS namespace включает владелец itzephir. Отдельный version tag выпускает бинарники. Откат сайта — предыдущий current; установщик позволяет pin прежней версии. Данные CLI не мигрируются.
