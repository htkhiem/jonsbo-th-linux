Jonsbo TH-240/360 temperature display on Linux
---

Minimal, no-UI Linux app to drive the temperature display on [Jonsbo TH-series](https://www.jonsbo.com/en/products/TH-360--.html) AIO water blocks. Basically functionally equivalent to their official Windows app, but for Linux and uses next to no RAM.

## Disclaimer

**Use this at your own risk.**. While this app only makes use of basic USB HID functionality (and therefore should be relatively safe compared to those having to use SMBus/i2c), there might still be a risk of malfunctioning or complete bricking of your AIO's display, stemming from possible differences between units and/or manufacturing batches. I have tested this on my own TH-360, but that does not necessarily translate to _every single_ Jonsbo TH-series unit sold out there.

As with GPLv3, this software is provided without warranty. The software author or license can not be held liable for any damages inflicted by the software.

## Instructions

1. Check whether your Jonsbo AIO is compatible by looking for its USB VID:PID in `lsusb`. This command might not be available out of the box depending on your distro. For example, on Arch Linux you'll need to install `usbutils` first.

    ```sh
    $ lsusb                                            
    Bus 001 Device 001: ID 1d6b:0002 Linux Foundation 2.0 root hub
    (...) 
    Bus 001 Device 006: ID 5131:2007 MSR MSR-101U Mini HID magnetic card reader
    (...)
    ```
    The TH-series water block displays should show up as some sort of "Mini HID magnetic card reader" (don't ask me, I don't know why either). If you see any such device in your `lsusb` result, especially with ID `5131:2007`, you're good to go. If you are **really sure** that your AIO is one of those two Jonsbo models but no such device exists, go to the FAQ section.

2. As a prerequisite, ensure you have a Rust toolchain installed:

    ``` sh
    rustup default stable
    ```
    
3. Clone & compile:

    ``` sh
    git clone https://github.com/htkhiem/jonsbo-th-linux.git
    cd jonsbo-th-linux
    cargo build --release
    ```
    
4. Install & register with `systemd` to run it on startup.

    ```sh
    chmod a+x install.sh
    sudo ./install.sh
    ```
    The display on the water block should now light up & display your CPU temperature, updated twice every second. **The display only has two digits & can only indicate up to 99°C.** Please be mindful of your fan profiles (not set via this app) and don't put yourself in an Anatoly Dyatlov situation.
    
    The above script installs for all users & requires `sudo` by default, but you can always modify the script to install into your user's folders like `~/bin/` and `~/.config/systemd/user/`, provided you have set up your `$PATH` accordingly.
    
## Uninstallation

    ```sh
    chmod a+x install.sh
    sudo ./install.sh --uninstall
    ```

## Manual start/stop

The above installation script will set up a `systemd` service named `jonsbo` that runs on boot. If desired, you may control it as with any other service:

    ```sh
    # Stop
    systemctl stop jonsbo
    # Disable autostart
    systemctl disable jonsbo
    # Start
    systemctl start jonsbo
    # Enable autostart & start immediately
    systemctl enable --now jonsbo
    ```

## What this thing does

1. Look for a thermal zone in `/sys/class/thermal` that's most likely the CPU package temperature sensor. By default we look for the first zone with type `x86_pkg_temp`. Read its temperature.
2. Look for the AIO display device itself, which is connected via an internal USB2 header. By default we're looking for one with PID:VID `5131:2007`. Different production batches may piggyback on different ICs and thus may have different IDs (and protocols, too). This app is only meant to work with `5131:2007`.
3. Ping said device with the acquired temperature. As with most HIDs it expects 64-byte packets, with 
   - The first two bytes being `0x01` and `0x02`, and
   - The **fourth** byte being the value to display.
   
   Other bytes didn't seem to do anything so they're set to zeros.
4. Sleep for half a second then repeat step 3. Steps 1 & 2 are not looped as there is no need to.

## FAQs

- **Q:** I don't see any such device in my `lsusb`/I know that particular device is my AIO display, but it has a different VID:PID. Can I use this app?

  **A:** Most probably NO. However, if you are pretty sure you have a TH-360/240 on hand with the exact same water block as seen on their website, you can try modifying `jonsbo.service` with your PID:VID. For example, if your device shows up with ID "1234:5678", edit the `ExecStart` line as follows:
  
  ```sh
  ExecStart=/usr/local/bin/jonsbo_th **1234:5678** x86_pkg_temp  # First parameter is the USB VID:PID to send data to. SETTING THIS TO AN INCOMPATIBLE DEVICE MAY DAMAGE IT. YOU HAVE BEEN WARNED.
  ```

- **Q:** How do I customise the temperature source?

  **A:** We make use of the `type` string in each thermal zone to settle on which to display. 
  
  First, list your current ones by running `cat /sys/class/thermal/*/type`. You might see something like this:
  
  ```sh
  $ cat /sys/class/thermal/*/type
  TFN1
  Fan
  x86_pkg_temp
  Processor
  Processor
  ...
  ```
  
  There might be duplicate values too (for example you may have one `Processor` for every CPU core). Only the first one with a given type is selected, so pinning to a specific CPU core isn't possible.
  
  Now, pick one of those values and edit them into the `ExecStart` line of `jonsbo.service`. For example with `TFN1`:
  
  ```sh
  ExecStart=/usr/local/bin/jonsbo_th 5131:2007 **TFN1**   # Second parameter determines what zone type to look for
  ```
  
- **Q:** Why Rust?
  **A:** I just like Rust.
