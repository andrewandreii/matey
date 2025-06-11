## Matey

A minimalist Material 3 theme generator for your wallpapers.

### Features
- No unnecessary features
- Uses caching for blazingly fast config generation
- Simple template syntax
- All template info in the same file (output file, renaming scheme)
- Uses only necessary dependencies

### Installation
The only available ways are with cargo or compiling from source at the moment.

#### With cargo
```sh
cargo install --git https://github.com/andrewandreii/matey.git

# Not available yet
cargo install matey
```

#### Compiling
```sh
git pull https://github.com/andrewandreii/matey.git
cd matey
cargo build -r
cargo install --path .
```

### Quick setup

Just follow these two simple steps:

#### 1) Write your templates in `~/.config/matey`
They usually define the output file, (optionally) a renaming scheme and the actual template:
```
#out "{CONFIG}/awesome-tool/config.conf"
#naming "camelCase"

foreach {
{name} = {color}
}
```

#### 2) Write a script to run matey and reload your tools
```sh
# -u will tell matey to use cache
matey -u $1

# Reload everything relevant
pkill --signal SIGUSR1 "helix"
pkill --signal SIGUSR1 "nvim"
swww img -t wipe $1
eww reaload
hyprctl reload
```

And you're done.

### Documentation

For a full description of the templates, consult the [wiki](https://github.com/andrewandreii/matey/wiki/Template-files) on github.

### Contributing

Contributions are welcome!

Feel free to open issues and make PRs (but make sure to run `cargo clippy` before)

### License

This project is under the [GPLv3](LICENSE) license.
