# Сборка DEB пакета для readual

## Требования

Для сборки deb пакета в WSL нужны следующие пакеты:

```bash
sudo apt-get update
sudo apt-get install -y debhelper cargo rustc libssl-dev pkg-config build-essential
```

## Сборка

1. Откройте WSL терминал
2. Перейдите в директорию проекта:
   ```bash
   cd /mnt/c/Users/Grin/Documents/Project/readual
   ```
3. Запустите скрипт сборки:
   ```bash
   ./build-deb.sh
   ```

Или вручную:

```bash
dpkg-buildpackage -b -uc -us
```

## Результат

После успешной сборки deb пакет будет находиться в родительской директории:
- `../readual_0.1.0-1_amd64.deb`

## Установка

```bash
sudo dpkg -i ../readual_0.1.0-1_amd64.deb
```

## Структура deb пакета

- `debian/control` - метаданные пакета
- `debian/changelog` - история изменений
- `debian/rules` - правила сборки
- `debian/compat` - версия debhelper

