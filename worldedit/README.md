# WorldEdit для Horizon

Мод **WorldEdit** (`worldedit`, версия `0.6.0`) для Minecraft-клиента/сервера **Horizon** (Rust).
Выделение областей, фигуры, шаблоны блоков, буфер обмена, история правок (undo/redo).

> Работает только в **творческом режиме** (`/gamemode creative`).

## Совместимость

| Параметр | Значение |
|---|---|
| Игра | `26.3` |
| Horizon (проверено на коммите) | `68ff4699847c591d3cb81f0af6b131a7e2239f3f` |
| Использованные хуки | 24 (только нативные hook-точки) |

Мод собирается на любом Horizon, где есть нужные нативные хуки.

## Установка

1. Соберите мод: `./build.sh` (получится `dist/worldedit-0.6.0.hnmod`).
2. Положите `.hnmod` в папку модов Horizon.

## Быстрый старт

- Жезл (wand): `minecraft:wooden_axe`. ЛКМ — первая точка, ПКМ — вторая.
- Или командами `/we pos1` и `/we pos2`.
- `/we set stone` — заполнить выделение.
- `/we undo` — отменить.

Паттерны: `stone`, `50%stone,dirt`, `0` = воздух. Направления: `north south east west up down`.

## Команды

Все команды начинаются с `/we`.

| Группа | Команды |
|---|---|
| Выделение | `wand`, `pos1`, `pos2`, `sel`, `desel`, `size`, `expand`, `contract`, `shift`, `outset`, `inset`, `chunk` |
| Правка области | `set`, `replace [!]<from> <to>`, `walls`, `faces`, `hollow`, `overlay`, `naturalize`, `smooth [n]`, `line`, `count` |
| Фигуры | `sphere`, `hsphere`, `cyl`, `hcyl`, `pyramid`, `hpyramid` |
| Буфер обмена | `copy`, `cut`, `paste [skipair]`, `rot <deg>`, `flip`, `clearclipboard`, `stack`, `move` |
| История | `undo [n]`, `redo [n]` |
| Вода/лава | `drain`, `fixwater`, `fixlava` |
| Настройки | `physics [on\|off]`, `limit [blocks\|radius\|history <n>\|reset]` |

Лимиты по умолчанию: 500 000 блоков за команду, радиус 128, история 20 правок (меняются через `/we limit`).

## Структура проекта

```
.
├── horizon-native.json            # манифест мода (id, версия, ревизия Horizon, хуки)
├── build.sh                       # сборка .hnmod
└── mixins/worldedit/
    ├── horizon-mixin.toml         # описание модулей и точек внедрения (injection)
    ├── we_core.rs                 # ядро: выделения, фигуры, паттерны, правки, буфер, сессии (без типов игры) + юнит-тесты
    ├── glue.rs                    # связка ядра с типами игры, парсинг команд /we
    ├── register.rs                # хук: регистрация команд
    ├── command.rs                 # хук: выполнение команды игрока
    ├── destroy_block.rs           # хук: ЛКМ жезлом (pos1)
    ├── right_click.rs             # хук: ПКМ жезлом (pos2)
    └── lifecycle.rs               # хук: закрытие мира — очистка сессий
```

`we_core.rs` намеренно не зависит от типов игры, поэтому его можно тестировать отдельно (`#[cfg(test)]` внутри файла).

## Лицензия

Добавьте файл `LICENSE` на свой выбор (MIT, Apache-2.0 и т.д.).
