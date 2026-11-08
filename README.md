<p align="center">
  <img src="logo.png" width="128px" alt="Логотип мода" />
</p>
<div align="center">

# 🪓 WorldEdit for Horizon

**A WorldEdit-style building toolkit for the [Horizon](https://playhorizon.dev) Minecraft client — native mod (`.hnmod`)**
**Мод в духе WorldEdit для клиента [Horizon](https://playhorizon.dev) — нативный мод (`.hnmod`)**

![version](https://img.shields.io/badge/version-0.8.0-blue) ![game](https://img.shields.io/badge/Horizon_Minecraft-26.3-green) ![hooks](https://img.shields.io/badge/native%20hooks-v24-lightgrey) ![mode](https://img.shields.io/badge/works%20in-creative-orange) ![ops](https://img.shields.io/badge/operator%20level%202%2B-orange)

</div>

---

## ✨ Features / Возможности

- 🪓 **Wooden axe wand** — left click = pos 1, right click = pos 2. Outside creative it is a normal axe. / **Топор-палочка**: ЛКМ — точка 1, ПКМ — точка 2. Вне creative это обычный топор.
- 🎨 **Creative only, operators only** — every command works only in `/gamemode creative` and only for operators (permission level 2); `/we limit` needs level 4. / **Только creative, только операторы** — все команды работают только в `/gamemode creative` и только для операторов (уровень прав 2), `/we limit` — только для уровня прав 4.
- 📍 **Exact corners** — `/we pos1 10,64,-5` and `/we pos2 10 64 -5` set a corner by coordinates; `/we hpos1` and `/we hpos2` take the block you look at. / **Точные углы** — `/we pos1 10,64,-5` и `/we pos2 10 64 -5` ставят угол по координатам; `/we hpos1` и `/we hpos2` берут блок, на который вы смотрите.
- 📐 **Selection tools** — `/we trim` cuts the selection down to the blocks that are really there, `/we distr` counts what is inside. / **Работа с выделением** — `/we trim` ужимает выделение до непустых блоков, `/we distr` показывает состав.
- 🕳️ **Digging** — `/we fill` and `/we fillr` fill the hole around you, `/we removeabove` and `/we removebelow` clear a column up or down. / **Копка** — `/we fill` и `/we fillr` заливают дыру вокруг вас, `/we removeabove` и `/we removebelow` чистят столб вверх или вниз.
- 🧱 **Fill, replace, walls, faces, hollow, overlay, naturalize, line** / заливка, замена, стены, грани, выдалбливание, слой сверху, натурализация, линия.
- ⚪ **Shapes** — spheres, cylinders, pyramids, solid or hollow (`hcyl`, `hsphere`, `hpyramid`). / **Фигуры** — шары, цилиндры, пирамиды, сплошные и полые.
- 🌊 **Terrain tools** — `drain`, `fixwater`, `fixlava`, `smooth`. / **Рельеф и жидкости**.
- 📋 **Clipboard** — copy, cut, paste, rotate, flip, stack, move. / **Буфер обмена**.
- ↩️ **Undo / redo** with configurable history. / **Отмена и возврат** с настраиваемой историей.
- 🎲 **Patterns** — `stone`, `0` (air), `stone,dirt`, `30%stone,dirt`. / **Паттерны**.
- 🔧 **In-game limits** — `/we limit` changes block, radius and history limits live. / **Лимиты** меняются прямо в игре.
- 🔇 **No item drops** — edits do not pop torches/flowers (toggle with `/we physics`). / **Без выпадения предметов** (переключается `/we physics`).

## 📦 Installation / Установка

#### Способ 1: Автоматический (Рекомендуемый)
1. Скопируйте ссылку на этот репозиторий GitHub из адресной строки браузера.
2. Откройте лаунчер **Horizon**, перейдите в раздел **Моды** -> **Установить по ссылке**.
3. Или просто вставьте ссылку в лаунчере от мода в GitHub, и он установится автоматически!
#### Method 1: Automatic (Recommended)
1. Copy the URL of this GitHub repository from your browser's address bar.
2. Open the **Horizon** launcher, go to **Mods** -> **Install from Link**.
3. Simply paste this GitHub link directly inside the launcher, and the mod will install automatically!
 #### Способ 2 \ Method 2 \ 
1. Download [worldedit_horizon](https://github.com/Kowge/WorldEdit_Horizon/releases). / Скачайте [worldedit_horizon](https://github.com/Kowge/WorldEdit_Horizon/releases).
2. Open the Horizon launcher → **Mods** → **Add mods** and pick the file. / Лаунчер Horizon → **Моды** → **Добавить моды** → выберите файл.
3. Enable **Settings → Native mods (experimental)** and press **Build** (the first build takes a while). / Включите **Настройки → Нативные моды** и нажмите **Собрать** (первая сборка долгая).
4. Launch the **Native mods** instance, enter a world and switch to creative: `/gamemode creative`. / Запустите сборку **Native mods**, войдите в мир и включите creative.

> ⚠️ Native mods are compiled into the game and have full access to your computer. Build only code you trust.
> ⚠️ Нативные моды компилируются в игру и имеют полный доступ к компьютеру. Собирайте только код, которому доверяете.

## 🚀 Quick start / Быстрый старт

```text
/gamemode creative
/we wand                    # get the axe / получить топор
(left click, right click)   # pick two corners / выбрать два угла
/we set stone               # fill / залить камнем
/we set 0                   # delete / удалить
/we expand 5                # grow where you look / расширить туда, куда смотрите
/we hcyl stone 5 3          # hollow cylinder r=5 h=3 / полый цилиндр
/we undo                    # undo / отмена
```

---

# 📖 Commands / Команды

All commands start with `/we`. They work only in creative mode (`/gamemode creative`) and for operators (permission level 2). `/we limit` needs permission level 4.
Все команды начинаются с `/we`. Работают только в режиме creative (`/gamemode creative`) и только для операторов (уровень прав 2). `/we limit` — только для уровня прав 4.

> RU: пишите с одним слешем — `/we set stone`. Двойной слеш (`//we set stone`) тоже выполнится (сервер срезает слэши), но чат подсвесит его как неизвестную команду.
> EN: write it with one slash — `/we set stone`. The double slash (`//we set stone`) runs as well (the server cuts the slashes off), but the chat box marks it as an unknown command.

## Выделение / Selection

| Команда / Command | RU | EN |
|---|---|---|
| `/we` | Справка | Show help |
| `/we wand` | Выдать деревянный топор: ЛКМ по блоку — точка 1, ПКМ — точка 2. Вне creative топор обычный | Give the wooden axe: left click = pos 1, right click = pos 2. Outside creative it is a normal axe |
| `/we pos1 [coordinates]`, `/we pos2 [coordinates]` | Поставить точку на блок, где вы стоите, или по координатам (`10,64,-5` или `10 64 -5`) | Set a corner at the block you stand on, or by coordinates (`10,64,-5` or `10 64 -5`) |
| `/we hpos1`, `/we hpos2` | Поставить точку на блок, на который вы смотрите | Set a corner at the block you are looking at |
| `/we pos` | Обе точки сразу: где вы стоите и блок, на который вы смотрите | Both corners at once: where you stand and the block you look at |
| `/we trim` | Ужать выделение до блоков, которые там действительно есть | Trim the selection down to the blocks that are really there |
| `/we distr` | Показать, из чего состоит выделение и сколько чего | Show what the selection holds and how much of it |
| `/we sel` (или `desel`) | Снять выделение | Clear the selection |
| `/we size` | Показать размеры и число блоков выделения | Show selection size and block count |
| `/we expand <n> [direction]` | Расширить выделение на n блоков. Без направления — туда, куда вы смотрите | Grow the selection by n blocks. No direction = the way you look |
| `/we contract <n> [direction]` | Сжать выделение с этой стороны | Shrink the selection from that side |
| `/we outset <n>` | Расширить выделение на n блоков со всех сторон | Grow the selection by n blocks on all sides |
| `/we inset <n>` | Сжать выделение на n блоков со всех сторон | Shrink the selection by n blocks on all sides |
| `/we chunk` | Выделить весь чанк, где вы стоите (по высоте от -64 до 319) | Select the whole chunk you stand in (height -64 to 319) |
| `/we shift <n> [direction]` | Сдвинуть выделение целиком | Move the selection itself |

Направления / Directions: `north south east west up down` (`n s e w u d`).

## Изменение блоков / Editing

| Команда / Command | RU | EN |
|---|---|---|
| `/we set <pattern>` | Заполнить выделение. `/we set 0` удаляет блоки | Fill the selection. `/we set 0` deletes the blocks |
| `/we replace <from> <to>` | Заменить одни блоки другими, напр. `replace dirt,grass_block stone` | Replace some blocks with others |
| `/we walls <pattern>` | Только 4 боковые стены | The 4 vertical sides only |
| `/we faces <pattern>` | Все 6 граней (пустая коробка) | All 6 sides (hollow box) |
| `/we replace !<blocks> <to>` | Знак `!` — заменить всё, кроме указанных блоков | A leading `!` replaces everything except the listed blocks |
| `/we overlay <pattern>` | Положить слой поверх самого верхнего блока каждой колонны | Put a layer on top of the highest block of each column |
| `/we naturalize` | Верх — трава, 3 слоя земли, ниже камень | Top becomes grass, 3 layers of dirt, stone below |
| `/we hollow` | Выдолбить выделение: оставить только стенки толщиной 1 | Hollow out the selection, keeping a 1-block shell |
| `/we line <pattern>` | Линия между точкой 1 и точкой 2 | A line between pos 1 and pos 2 |
| `/we drain` | Убрать воду, лаву и водные растения в выделении | Remove water, lava and water plants in the selection |
| `/we fixwater` | Превратить текущую воду в выделении в источники | Turn flowing water in the selection into source blocks |
| `/we fixlava` | То же для лавы | The same for lava |
| `/we smooth [n]` | Сгладить рельеф в выделении, n проходов (1–50, по умолчанию 1) | Smooth the terrain in the selection, n passes (1–50, default 1) |
| `/we count <blocks>` | Посчитать блоки в выделении | Count blocks in the selection |
| `/we fill <pattern> <radius> [depth]` | Залить дыру вокруг вас: каждый столбец до первого блока | Fill the hole around you: each column down to the first block |
| `/we fillr <pattern> <radius> [depth]` | То же, но весь воздух внутри шара | The same, but the whole air of the ball |
| `/we removeabove [size] [height]` | Убрать блоки над вами (квадрат `size` × `height`) | Remove the blocks above you (a `size` × `height` square) |
| `/we removebelow [size] [height]` | Убрать блоки под вами | Remove the blocks below you |

## Фигуры вокруг ног / Shapes at your feet

Не требуют выделения. / No selection needed.

| Команда / Command | RU | EN |
|---|---|---|
| `/we sphere <pattern> <radius> [true]` | Шар; `true` — полый | Ball; `true` = hollow |
| `/we hsphere <pattern> <radius>` | Полая сфера | Hollow sphere |
| `/we cyl <pattern> <radius> [height] [true]` | Цилиндр (высота по умолчанию 1); `true` — полый, с крышками | Cylinder (height defaults to 1); `true` = hollow with caps |
| `/we hcyl <pattern> <radius> [height]` | Полый цилиндр без крышек; высота 1 даёт кольцо | Hollow cylinder, open top and bottom; height 1 = ring |
| `/we pyramid <pattern> <size>` | Пирамида из `size` слоёв | Pyramid with `size` layers |
| `/we hpyramid <pattern> <size>` | Полая пирамида | Hollow pyramid |

## Буфер обмена / Clipboard

| Команда / Command | RU | EN |
|---|---|---|
| `/we copy` | Скопировать выделение (относительно вас) | Copy the selection (relative to you) |
| `/we cut` | Скопировать и удалить | Copy and delete |
| `/we paste [true]` | Вставить у ног; `true` — без воздуха | Paste at your feet; `true` = skip air |
| `/we rot <degrees>` | Повернуть буфер (кратно 90) | Rotate the clipboard (multiples of 90) |
| `/we flip [direction]` | Отразить буфер | Flip the clipboard |
| `/we clearclipboard` | Очистить буфер | Clear the clipboard |
| `/we stack <n> [direction]` | Повторить выделение n раз | Repeat the selection n times |
| `/we move <n> [direction]` | Переместить блоки и выделение | Move blocks and the selection |

## История / History

| Команда / Command | RU | EN |
|---|---|---|
| `/we undo [n]` | Отменить n последних действий (до 20) | Undo n last edits (up to 20) |
| `/we redo [n]` | Вернуть отменённое | Redo undone edits |

## Лимиты / Limits

| Команда / Command | RU | EN |
|---|---|---|
| `/we limit` | Показать текущие лимиты | Show current limits |
| `/we limit blocks <n>` | Макс. блоков за команду (1–20 000 000) | Max blocks per command (1–20,000,000) |
| `/we limit radius <n>` | Макс. радиус/размер фигур (1–1024) | Max shape radius/size (1–1024) |
| `/we limit history <n>` | Сколько действий хранит undo (1–200) | How many edits undo keeps (1–200) |
| `/we limit reset` | Вернуть значения по умолчанию | Restore defaults |

RU: лимиты общие для всех игроков и сбрасываются при перезапуске игры; меняет их только оператор уровня 4. EN: limits are shared by all players and reset when the game restarts; only an operator of level 4 changes them.

## Физика / Physics

| Команда / Command | RU | EN |
|---|---|---|
| `/we physics` | Показать режим физики | Show the physics mode |
| `/we physics off` | (по умолчанию) соседние блоки не обновляются: после `/we set 0` не выпадают предметы с факелов, цветов и т. п. | (default) neighbours are not updated: no dropped items from torches, flowers etc. after `/we set 0` |
| `/we physics on` | Обычные обновления: висящие блоки отваливаются и дают предметы | Normal updates: dependent blocks pop off and drop items |

## Примечания / Notes

RU: `drain` не убирает воду из «залитых» блоков (waterlogged) — например, ступенек. `smooth` работает по верхнему не-воздушному блоку каждой колонны выделения. `regen` (перегенерация области) не реализован: у модов нет доступа к генератору мира.
EN: `drain` does not remove water inside waterlogged blocks (e.g. stairs). `smooth` works on the highest non-air block of each selection column. `regen` (regenerating an area) is not implemented: mods have no access to the world generator.

## Паттерны и лимиты / Patterns and limits

- **RU:** `stone` — один блок; `0` — воздух; `stone,dirt` — поровну; `30%stone,dirt` — 30% камня, остальное земля. Имена без `minecraft:` допустимы.
- **EN:** `stone` = one block; `0` = air; `stone,dirt` = even mix; `30%stone,dirt` = 30% stone, the rest dirt. The `minecraft:` prefix is optional.
- **RU:** по умолчанию за одну команду не больше 500 000 блоков, радиус и размер фигур — до 128, история — 20 действий и сбрасывается при выходе из мира. Меняет лимиты только оператор уровня 4 (`/we limit`).
- **EN:** by default at most 500,000 blocks per command, shape radius and size up to 128, history keeps 20 edits and resets when you leave the world. Only an operator of level 4 changes the limits (`/we limit`).

---

## ⚠️ Known limitations / Ограничения

- Selection grid is not implemented (no line-drawing API in Horizon). / Сетка выделения не реализована (в Horizon нет API линий).
- `regen`, brushes and schematics are not implemented. / Не реализованы `regen`, кисти и схематики.
- `rotate` is named `/we rot` because vanilla already has `/rotate`. / Поворот буфера — `/we rot`, т. к. `/rotate` занят ванилью.
- Block orientation (`facing`) is not rotated by `rot` / `flip`. / `rot` и `flip` не поворачивают ориентацию блоков.
- Limits, physics mode and history are not saved between game launches. / Лимиты, режим физики и история не сохраняются между запусками.

## 🛠 Building / Сборка

Built as a Horizon native mod (mixin bundle). Logic lives in `we_core.rs` (pure Rust, unit-tested with `cargo test`), game glue in `glue.rs`.
Нативный мод Horizon (mixin-бандл). Логика — в `we_core.rs` (чистый Rust, тесты `cargo test`), связка с игрой — в `glue.rs`.

```bash
python3 native/tools/native.py check worldedit     # check against Horizon source
python3 native/tools/native.py package worldedit   # produce worldedit-<version>.hnmod
```

## 📄 License / Лицензия

MIT
