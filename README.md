i3rustus
========

`i3status` is my own  implementation of [i3status](https://github.com/i3/i3status).
The goal is that I replace i3status with this and build custom stats that are not available in i3status.

This project is not intended for other people to use.
It's one of my rust learning projects,
where I am getting to know rust better.

Configuration
-------------

The config file is located at `$HOME/config/i3rustus/config`.

Example:

```
wlan_device_name = wlp3s0
ethernet_device_name = enp5s0
battery_device_name = BAT0
brightness_device_name = amdgpu_bl1
timezone_file = /etc/timezone
order = wlan, ethernet, battery, brightness, cpu_load, cpu_percentage, memory, disk, time
```

Installation
------------

Fedora:

```
dnf copr enable blinxen/tools
dnf install fedtools
```

License
-------

The source code is primarily distributed under the terms of the MIT License.
See LICENSE for details.
