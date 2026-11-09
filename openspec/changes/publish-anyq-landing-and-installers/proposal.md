# Proposal

## Why

Для знакомства с anyq нужен отдельный лендинг и установка готового бинарника без Rust toolchain. Проект должен появиться в каталоге itzephir.com, а редактируемый макет — в существующем OpenDesign.

## What Changes

- Статический лендинг anyq.itzephir.com в этом репозитории, с основой itzephir, двумя темами и мобильной композицией.
- Shell и PowerShell установщики с определением платформы, SHA-256 и системной установкой по умолчанию; явный пользовательский режим сохраняется.
- Нативная сборка release-артефактов для macOS/Linux/Windows и публикация полных релизов через GitHub Releases.
- Воспроизводимый пакет для OpenDesign и контейнер лендинга для существующего project gateway.
- Запись anyq в каталоге через /dev, без изменения кода портфолио.
- Автоматический dev-стенд dev.anyq.itzephir.com из PR и production из main; проектный ingress самостоятельно выбирает стенд по Host.

## Capabilities

### New Capabilities

- `binary-distribution`: выпуск и проверенная установка готовых бинарников.
- `project-website`: лендинг, редактируемый макет и публикация через существующий gateway.

### Modified Capabilities

Нет. Доменные контракты очереди и обязательный CI остаются прежними.

## Impact

Новые website/, install/, release/deploy tooling и GitHub workflows. Runtime anyq не получает новых зависимостей. Дизайн использует лицензированные локальные IBM Plex Sans/Mono. Публичные файлы не содержат операторских адресов и credentials. Слияние PR и выпуск версии остаются отдельными явными действиями.
