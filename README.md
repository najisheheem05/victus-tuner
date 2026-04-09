# Victus Tuner (Rust)

Control the keyboard RGB lighting on **HP Victus laptops** directly from Linux using a **native Rust program**.

Victus-tuner allows changing keyboard colors and running lighting effects **without Windows or OMEN Gaming Hub** by writing RGB values directly to the laptop’s **Embedded Controller (EC)**.

This project is a **Rust rewrite of the original Python implementation**, providing:

- lower CPU usage
- faster updates
- a single compiled binary
- no Python dependency

---

# Features

- Change keyboard RGB directly from Linux
- No Windows software required
- Lightweight native binary
- Automatic EC access setup
- Background lighting effects
- Automatically replaces previously running effects
- PID-based worker management
- No Python required

Supported lighting modes:

- Static colors
- Rainbow
- Breathing effect
- Alternate between two colors
- Fade between two colors (bidirectional)
- Ambient (screen-reactive lighting)

---

# How It Works

Keyboard RGB values are stored inside **Embedded Controller memory**.

RGB values begin at **offset `0x08`**.

Example values discovered during testing:

| Color | EC Bytes   |
| ----- | ---------- |
| Red   | `e4 00 00` |
| Green | `00 e4 00` |
| Blue  | `00 00 e4` |

The program writes RGB values directly to:

```
/sys/kernel/debug/ec/ec0/io
```

---

# Requirements

- Linux
- Rust toolchain
- Root privileges
- `ec_sys` kernel module

The program **automatically loads `ec_sys` and mounts debugfs when required**.

---

# Installation

Clone the repository:

```bash
git clone https://github.com/<username>/victus-tuner
cd victus-tuner
```

Build the binary:

```bash
cargo build --release
```

Binary location:

```
target/release/victuner
```

Optional: install globally

```bash
sudo cp target/release/victuner /usr/local/bin/victuner
```

Then run:

```bash
sudo victuner rgb red
```

---

# Preset Colors

| Color       | Command                           |
| ----------- | --------------------------------- |
| red         | `sudo victuner rgb red`           |
| green       | `sudo victuner rgb green`         |
| blue        | `sudo victuner rgb blue`          |
| yellow      | `sudo victuner rgb yellow`        |
| cyan        | `sudo victuner rgb cyan`          |
| purple      | `sudo victuner rgb purple`        |
| neon-purple | `sudo victuner rgb neon-purple`   |
| white       | `sudo victuner rgb white`         |
| off         | `sudo victuner rgb off`           |

Example:

```bash
sudo victuner rgb neon-purple
```

---

# Custom RGB Values

Set an arbitrary color using raw RGB values (0–255).

```bash
sudo victuner rgb 255 128 0
```

You can use this in effects too

```bash
sudo victuner rgb fade 255 128 0 0 255 0
```

---

# Read Current Color

Display the RGB value currently stored in EC.

```bash
sudo victuner rgb current
```

Example output:

```
Current RGB: 255 0 0
```

---

# Lighting Effects

Lighting effects run **in the background automatically**.

Starting a new effect **stops the previous one automatically**.

The running worker PID is stored in:

```
/tmp/victus-rgb.pid
```

---

## Rainbow

Cycle smoothly through all colors.

```bash
sudo victuner rgb rainbow
```

---

## Breathing Effect

Fade brightness in and out.

```bash
sudo victuner rgb breathe red
```

Example:

```bash
sudo victuner rgb breathe neon-purple
```

---

## Alternate Between Two Colors

Switch between two colors repeatedly.

```bash
sudo victuner rgb alternate red blue
```

---

## Fade Between Two Colors

Transition between two colors **in both directions**.

```
color1 → color2 → color1 → repeat
```

Example:

```bash
sudo victuner rgb fade red blue
```

Example transition:

```
red → purple → blue → purple → red → ...
```

---

## Ambient Mode

Match the keyboard color to the **dominant color on screen** in real time (~30 FPS).

The effect captures the screen using [`grim`](https://sr.ht/~emersion/grim/), quantizes the image to a single dominant color with [ImageMagick](https://imagemagick.org/), then applies vibrance boosting and smooth transitions in HSV color space before writing the result to the keyboard.

**Additional requirements:**

- Wayland compositor 
(I use Hyprland, If I could get an x11 system I would try to implement for it also, If anyone's interested in doing x11 implementation, I would be happy to merge it )
- `grim` (Wayland screenshot tool)
- ImageMagick (`magick` v7 or `convert` v6)

**Usage:**

```bash
sudo -E victuner rgb ambient
```

> **Note:** The `-E` flag preserves the `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR` environment variables needed for screen capture. If those variables are not available, the program will attempt to auto-detect them from the real user's runtime directory.

**With boosting vibrancy of color:**

```bash
sudo -E victuner rgb ambient 3
```

| Level | Effect                                |
| ----- | ------------------------------------- |
| 0     | Raw screen color (default)            |
| 1     | Slightly more saturated and brighter  |
| 2     | Noticeably vivid                      |
| 3     | Strong vibrance boost                 |
| 4–5   | Maximum saturation and brightness     |

Sometimes using vibrancy boost seems changing color by little(like blue to purple), so I would prefer to keep vibrancy low or not using it.

---

# Stop Effects

Stop any running lighting effect.

```bash
sudo victuner rgb stop
```

---

# Supported Hardware

Tested on:

- **HP Victus 16**

Other Victus and Omen laptops may work if they use the same EC RGB layout.

---

# Warning

This tool writes directly to **Embedded Controller registers**.

Incorrect values may:

- freeze the keyboard controller
- crash the EC
- require a hard reboot

Use at your own risk.

---

# License

MIT License
