# Basic Layout
A simple layout generator for the [river](https://codeberg.org/river/river) window manager with one layout per tag. Supported layouts are:

- Tile: Main window/windows to the left and the rest as rows to the right.
- Column: All windows get equally spaced columns, from left to right.
- Rows: All windows get equally spaced rows, from top to bottom.
- Centered Master: Main window/windows get the center of the screen and the rest surround it to the left and right.
- Dwindle: Each new window splits the remaining space in half.

## Commands

```
gap [int]
```

Set the gap between windows. (default: ```8```)

---

```
padding [int]
```

Set the gap between the windows and the edge of the screen. (default: ```8```)

---

```
layout [layout]
```

Set the current layout of the tag. Supported values are: ```tile```, ```column```, ```rows```, ```centered-master``` and ```dwindle```. (default: ```tile```)

---

```
main-ratio [ratio]
```

Set the ratio of the main window/windows to the rest of the windows. Supports both absolute values such as ```0.6``` and relative values such as ```-0.1``` or ```+0.1```. (default: ```0.5```)

---

```
main-count [count]
```

Set the amount of windows to be contained in the main area of the current layout. Supports both absolute values such as ```3``` and relative values such as ```-1``` or ```+1```. The minimum value is 1. (default: ```1```)
