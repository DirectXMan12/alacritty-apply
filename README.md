# alacritty-apply

Apply configuration to alacritty dynamically, from Alacritty TOML files.

Requires alacritty after https://github.com/alacritty/alacritty/commit/bd4906722a1a026b01f06c94c33b13ff63a7e044,
which switches to TOML files for config.

## Usage

See `--help` for more details, but basically:

```shell
# all windows, from file
alap ~/.config/alacritty/dark-mode.toml
# just this window, from stdin
print "[window.padding]\nx = 10\ny=10" | alap --window=self -
```

## What's Not Working

Complex arrays (e.g. mouse.bindings) might not work.
Need to figure out how alacritty handles them, if it does
at all.
