<p align="center">
  <img src="logo.png" width="128px" alt="WorldEdit for Horizon" />
</p>
<div align="center">

# 🪓 WorldEdit for Horizon

**A WorldEdit-style building toolkit for the [Horizon](https://playhorizon.dev) Minecraft client — native mod (`.hnmod`)**

![version](https://img.shields.io/badge/version-1.0.0-blue) ![game](https://img.shields.io/badge/Horizon_Minecraft-26.3-green) ![hooks](https://img.shields.io/badge/native%20hooks-v58-lightgrey) ![mode](https://img.shields.io/badge/works%20in-creative-orange) ![ops](https://img.shields.io/badge/operator%20level%202%2B-orange)

</div>

---

<details open>
<summary>🌐 <b>English</b> — features, installation, quick start</summary>

## ✨ Features

- 🪓 **Wooden axe wand** — left click = pos 1, right click = pos 2. Outside creative it is a normal axe.
- 🎨 **Creative only, operators only** — every command works only in `/gamemode creative` and only for operators (permission level 2); `/we limit` needs level 4.
- 📍 **Exact corners** — `/we pos1 10,64,-5` and `/we pos2 10 64 -5` set a corner by coordinates; `/we hpos1` and `/we hpos2` take the block you look at.
- 📐 **Selection tools** — `/we trim` cuts the selection down to the blocks that are really there, `/we distr` counts what is inside.
- 🕳️ **Digging** — `/we fill` and `/we fillr` fill the hole around you, `/we removeabove` and `/we removebelow` clear a column up or down.
- 🧱 **Fill, replace, walls, faces, hollow, overlay, naturalize, line.**
- ⚪ **Shapes** — spheres, cylinders, pyramids, solid or hollow (`hcyl`, `hsphere`, `hpyramid`).
- 🔷 **Shapes for the selection** — `/we sel sphere 5`, `/we sel cylinder 3 4`, `/we sel ellipsoid 4 2 4`: the selection becomes a ball, a cylinder or an ellipsoid instead of a box.
- 🟥 **The selection is drawn for you** — a red frame around the box and a grid inside it, one cell per block. A round shape is drawn as the blocks it covers, so the outline stands on the blocks the command would change (like F3+B). The lines are drawn over everything, so the terrain does not hide the selection.
- 🔲 **Your own view** — `/we grid on` and `/we grid off` turn the grid inside the selection off and on; the frame stays. The setting is yours alone.
- 🗺 **Chunks** — `/we chunkinfo` tells what the chunk under you holds, `/we listchunks` lists the chunks the selection covers.
- 🌊 **Terrain tools** — `drain`, `fixwater`, `fixlava`, `smooth`.
- 📋 **Clipboard** — copy, cut, paste, rotate, flip, stack, move.
- ↩️ **Undo / redo** with configurable history.
- 🎲 **Patterns** — `stone`, `0` (air), `stone,dirt`, `30%stone,dirt`.
- 🔧 **In-game limits** — `/we limit` changes block, radius and history limits live.
- 🔇 **No item drops** — edits do not pop torches/flowers (toggle with `/we physics`).

## 📦 Installation

#### Method 1: Automatic (Recommended)

1. Copy the URL of this GitHub repository from your browser's address bar.
2. Open the **Horizon** launcher, go to **Mods** -> **Install from Link**.
3. Simply paste this GitHub link directly inside the launcher, and the mod will install automatically!

#### Method 2: Manual

1. Download [worldedit_horizon](https://github.com/Kowge/WorldEdit_Horizon/releases).
2. Open the Horizon launcher → **Mods** → **Add mods** and pick the file.
3. Enable **Settings → Native mods (experimental)** and press **Build** (the first build takes a while).
4. Launch the **Native mods** instance, enter a world and switch to creative: `/gamemode creative`.

> ⚠️ Native mods are compiled into the game and have full access to your computer. Build only code you trust.

## 🚀 Quick start

```text
/gamemode creative
/we wand                    # get the axe
(left click, right click)   # pick two corners
/we set stone               # fill
/we set 0                   # delete
/we expand 5                # grow where you look
/we hcyl stone 5 3          # hollow cylinder r=5 h=3
/we sel sphere 5            # a round selection
/we undo                    # undo
```

</details>

<details>
<summary>🌐 <b>Русский</b> — возможности, установка, быстрый старт</summary>

## ✨ Возможности

- 🪓 **Топор-палочка** — ЛКМ — точка 1, ПКМ — точка 2. Вне creative это обычный топор.
- 🎨 **Только creative, только операторы** — все команды работают только в `/gamemode creative` и только для операторов (уровень прав 2), `/we limit` — только для уровня прав 4.
- 📍 **Точные углы** — `/we pos1 10,64,-5` и `/we pos2 10 64 -5` ставят угол по координатам; `/we hpos1` и `/we hpos2` берут блок, на который вы смотрите.
- 📐 **Работа с выделением** — `/we trim` ужимает выделение до непустых блоков, `/we distr` показывает состав.
- 🕳️ **Копка** — `/we fill` и `/we fillr` заливают дыру вокруг вас, `/we removeabove` и `/we removebelow` чистят столб вверх или вниз.
- 🧱 **Заливка, замена, стены, грани, выдалбливание, слой сверху, натурализация, линия.**
- ⚪ **Фигуры** — шары, цилиндры, пирамиды, сплошные и полые (`hcyl`, `hsphere`, `hpyramid`).
- 🔷 **Форма выделения** — `/we sel sphere 5`, `/we sel cylinder 3 4`, `/we sel ellipsoid 4 2 4`: выделение становится шаром, цилиндром или эллипсоидом вместо коробки.
- 🟥 **Выделение рисуется** — красная рамка вокруг коробки и сетка внутри, ячейка ровно в один блок. Круглая фигура рисуется своими блоками, так что контур стоит на тех блоках, которые изменит команда (как F3+B). Линии рисуются поверх всего — рельеф не прячет выделение.
- 🔲 **Своё отображение** — `/we grid on` и `/we grid off` гасят и возвращают внутреннюю сетку выделения; рамка остаётся. Настройка личная.
- 🗺 **Чанки** — `/we chunkinfo` расскажет о чанке под вами, `/we listchunks` покажет чанки выделения.
- 🌊 **Рельеф и жидкости** — `drain`, `fixwater`, `fixlava`, `smooth`.
- 📋 **Буфер обмена** — копировать, вырезать, вставить, повернуть, отразить, повторить, переместить.
- ↩️ **Отмена и возврат** с настраиваемой историей.
- 🎲 **Паттерны** — `stone`, `0` (воздух), `stone,dirt`, `30%stone,dirt`.
- 🔧 **Лимиты** меняются прямо в игре — `/we limit`.
- 🔇 **Без выпадения предметов** — правки не роняют факелы и цветы (переключается `/we physics`).

## 📦 Установка

#### Способ 1: Автоматический (рекомендуемый)

1. Скопируйте ссылку на этот репозиторий GitHub из адресной строки браузера.
2. Откройте лаунчер **Horizon**, перейдите в раздел **Моды** -> **Установить по ссылке**.
3. Или просто вставьте ссылку в лаунчере от мода на GitHub, и он установится автоматически!

#### Способ 2: Вручную

1. Скачайте [worldedit_horizon](https://github.com/Kowge/WorldEdit_Horizon/releases).
2. Лаунчер Horizon → **Моды** → **Добавить моды** → выберите файл.
3. Включите **Настройки → Нативные моды** и нажмите **Собрать** (первая сборка долгая).
4. Запустите сборку **Native mods**, войдите в мир и включите creative: `/gamemode creative`.

> ⚠️ Нативные моды компилируются в игру и имеют полный доступ к компьютеру. Собирайте только код, которому доверяете.

## 🚀 Быстрый старт

```text
/gamemode creative
/we wand                    # получить топор
(ЛКМ, ПКМ)                  # выбрать два угла
/we set stone               # залить
/we set 0                   # удалить
/we expand 5                # расширить туда, куда смотрите
/we hcyl stone 5 3          # полый цилиндр r=5 h=3
/we sel sphere 5            # круглое выделение
/we undo                    # отмена
```

</details>

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
| `/we sel <фигура> [размеры]` | Форма выделения: `cube`, `sphere`, `cylinder`, `ellipsoid`. С числами — объём сразу вокруг вас, без чисел — по углам `pos1`/`pos2` | Shape of the selection: `cube`, `sphere`, `cylinder`, `ellipsoid`. With sizes the shape appears around you at once, without them it follows the `pos1`/`pos2` corners |
| `/we grid on` / `/we grid off` | Внутренняя сетка выделения: вкл или выкл. Рамка остаётся всегда. Настройка личная | The grid inside the selection: on or off. The frame always stays. Your setting alone |
| `/we size` | Показать размеры и число блоков выделения | Show selection size and block count |
| `/we expand <n> [direction]` | Расширить выделение на n блоков. Без направления — туда, куда вы смотрите | Grow the selection by n blocks. No direction = the way you look |
| `/we contract <n> [direction]` | Сжать выделение с этой стороны | Shrink the selection from that side |
| `/we outset <n>` | Расширить выделение на n блоков со всех сторон | Grow the selection by n blocks on all sides |
| `/we inset <n>` | Сжать выделение на n блоков со всех сторон | Shrink the selection by n blocks on all sides |
| `/we chunk` | Выделить весь чанк, где вы стоите (по высоте от -64 до 319) | Select the whole chunk you stand in (height -64 to 319) |
| `/we shift <n> [direction]` | Сдвинуть выделение целиком | Move the selection itself |

Направления / Directions: `north south east west up down` (`n s e w u d`).

## Форма выделения / Selection shape

`/we sel <фигура> [размеры]` меняет форму выделения: обычная коробка (`cube`) или круглая фигура внутри неё.

- **RU:** с числами объём появляется вокруг вас: `/we sel sphere 5` — шар радиуса 5, `/we sel cylinder 3 4` — цилиндр радиуса 3 и высотой 4, `/we sel ellipsoid 4 2 4` — эллипсоид с такими радиусами, `/we sel cube 5` — куб 5×5×5. Без чисел форма ждёт углов: `/we pos1` и `/we pos2`, потом `/we sel sphere`.
- **EN:** with sizes the volume appears around you: `/we sel sphere 5` — a ball of radius 5, `/we sel cylinder 3 4` — a cylinder of radius 3 and height 4, `/we sel ellipsoid 4 2 4` — an ellipsoid with those radii, `/we sel cube 5` — a 5×5×5 cube. Without sizes the shape waits for the corners: `/we pos1` and `/we pos2`, then `/we sel sphere`.

Как считаются размеры / How the sizes work:

- **RU:** куб — сторона; сфера — радиус, центр в первом углу; цилиндр — радиус по горизонтали и высота рядами, низ в первом углу; эллипсоид — три радиуса, центр в первом углу.
- **EN:** a cube is its side; a sphere is a radius with its middle in the first corner; a cylinder is a horizontal radius and a height in rows, its bottom in the first corner; an ellipsoid is three radii with its middle in the first corner.

## Отрисовка выделения / How the selection is drawn

- **RU:** выделение рисуется на клиенте: красная рамка вокруг коробки (`#CCFF0000`) и сетка внутри (`#CCAA0000`), ячейка ровно в один блок при любом размере выделения. Круглая фигура рисуется своими блоками, квадрат за квадратом, — контур стоит ровно на тех блоках, которые изменит команда (как F3+B). Фигура с очень большим числом блоков рисуется тремя сечениями через её середину, чтобы оси не разъезжались. Линии рисуются поверх всего, поэтому рельеф не прячет выделение. `/we grid off` гасит только внутренние линии сетки, рамка остаётся; настройка своя у каждого игрока и переживает новое выделение.
- **EN:** the selection is drawn on the client: a red frame around the box (`#CCFF0000`) and a grid inside it (`#CCAA0000`), one cell per block whatever the size of the selection is. A round shape is drawn as the blocks it covers, square by square, so the outline stands exactly on the blocks the command would change (like F3+B). A shape with too many blocks is drawn as its three sections through the middle, so its axes do not slip apart. The lines are drawn over everything, so the terrain does not hide the selection. `/we grid off` turns only the inner grid lines off; the frame stays. The setting is yours alone and survives a new selection.

## Чанки / Chunks

| Команда / Command | RU | EN |
|---|---|---|
| `/we chunkinfo` | Чанк, где вы стоите, и сколько в нём блоков (высота -64..319) | The chunk you stand in and how many blocks it holds (height -64..319) |
| `/we listchunks` | Список чанков, которые покрывает выделение | The chunks the selection covers |

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
| `/we redo [n]` | Вернуть отменённые | Redo undone edits |

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

- `regen`, brushes and schematics are not implemented. / Не реализованы `regen`, кисти и схематики.
- `rotate` is named `/we rot` because vanilla already has `/rotate`. / Поворот буфера — `/we rot`, т. к. `/rotate` занят ванилью.
- Block orientation (`facing`) is not rotated by `rot` / `flip`. / `rot` и `flip` не поворачивают ориентацию блоков.
- Limits, physics mode and history are not saved between game launches. / Лимиты, режим физики и история не сохраняются между запусками.

## 🛠 Building / Сборка

Built as a Horizon native mod (mixin bundle). Logic lives in `we_core.rs` (pure Rust, unit-tested with `cargo test`), game glue in `glue.rs`, and the drawing of the selection in `geometry.rs` and `render.rs`.
Нативный мод Horizon (mixin-бандл). Логика — в `we_core.rs` (чистый Rust, тесты `cargo test`), связка с игрой — в `glue.rs`, отрисовка выделения — в `geometry.rs` и `render.rs`.

```bash
python3 native/tools/native.py check worldedit     # check against Horizon source
python3 native/tools/native.py package worldedit   # produce worldedit-<version>.hnmod
```

## 📄 License / Лицензия

MIT
