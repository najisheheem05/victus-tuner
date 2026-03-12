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
target/release/victus-rgb-tuner
```

Optional: install globally

```bash
sudo cp target/release/victus-rgb-tuner /usr/local/bin/victus-rgb
```

Then run:

```bash
sudo victus-rgb red
```

---

# Preset Colors

| Color       | Command                       |
| ----------- | ----------------------------- |
| red         | `sudo victus-rgb red`         |
| green       | `sudo victus-rgb green`       |
| blue        | `sudo victus-rgb blue`        |
| yellow      | `sudo victus-rgb yellow`      |
| cyan        | `sudo victus-rgb cyan`        |
| purple      | `sudo victus-rgb purple`      |
| neon-purple | `sudo victus-rgb neon-purple` |
| white       | `sudo victus-rgb white`       |
| off         | `sudo victus-rgb off`         |

Example:

```bash
sudo victus-rgb neon-purple
```

---

# Read Current Color

Display the RGB value currently stored in EC.

```bash
sudo victus-rgb current
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
sudo victus-rgb rainbow
```

---

## Breathing Effect

Fade brightness in and out.

```bash
sudo victus-rgb breathe red
```

Example:

```bash
sudo victus-rgb breathe neon-purple
```

---

## Alternate Between Two Colors

Switch between two colors repeatedly.

```bash
sudo victus-rgb alternate red blue
```

---

## Fade Between Two Colors

Transition between two colors **in both directions**.

```
color1 → color2 → color1 → repeat
```

Example:

```bash
sudo victus-rgb fade red blue
```

Example transition:

```
red → purple → blue → purple → red → ...
```

---

# Stop Effects

Stop any running lighting effect.

```bash
sudo victus-rgb stop
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
