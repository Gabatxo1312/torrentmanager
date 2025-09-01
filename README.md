# TorrentManager

> [!CAUTION]
> This branch is not usable. Uploads are not yet implemented.

TorrentManager is a torrenting web interface to manage your public archives. By using symlinks from the content directory to the torrent download directory, you can filter and rename exposed files without breaking the torrent structure for seeding.

## Planned features

TorrentManager is not feature-complete yet. This branch is the third iteration of the project, which should prove more mature to continue development. For history's sake:

- first prototype: PHP interface + bash/python processing scripts, only used for uploading torrents to qBittorrent (no follow-up)
- second iteration: Rust/rocket + bash/python processing scripts, allows viewing torrents from qBittorrent and filtering stuck torrents
- third iteration (this branch): Rust/axum

In the future, TorrentManager will become federated and allow you to find content from your friends to help and distribute it. For example, subscribing to a hypothetical [media.ccc.de](https://media.ccc.de/) instance would help seeding their video files and automatically importing them into your local media library.

- [ ] Torrent backend integrations
  - [x] qBittorrent v5.0.x/v5.1.x
  - [ ] Transmission
  - [ ] rqbit
- [ ] Torrent categories (video, iso, etc...) for placement in different folders
- [ ] Torrent meta files (eg. associated subtitles)
- [ ] Federation
  - [ ] Following new imports on other instances (RSS/ActivityPub)
  - [ ] Importing from other instances
  - [ ] Auto-importing from other instances (social redundancy/backup)

## Usage

### Command-line usage

For the moment: `torrentmanager` will start the HTTP server.

The `-c/--config` flag will specify a config file to load that's not the default.

### Settings

Settings are defined in the `$XDG_CONFIG_DIR/torrentmanager/config.toml` (usually `~/.config/torrentmanager/config.toml`).

# Architecture

TorrentManager is now a single binary embedding all assets/templates: the only external dependency is a supported torrent client (only qBittorrent v5.0+ at the moment).

The following files/folders are used:

- `$XDG_CONFIG_DIR/torrentmanager/config.toml`: the daemon configuration file
- `$XDG_DATA_DIR/torrentmanager/uploads`: folder where a copy of the user-uploaded magnets/torrents is stored

# Request lifecyle

When a request arrives:

- it's encapsulated in a [timing middleware](src/middleware/timing.rs), which will:
  - add a `x-generation-time: XXms` HTTP header to all responses
  - replace the magic string `__GENERATION_TIME__` in any HTML response
- on most routes, global `AppStateContext` is computed, containing:
  - the username of the logged-in user, or None
  - free space calculation for the configured `media_dir`

Additionally, request handlers which interact with the torrent backend check that it's alive and credentials are good, preventing you from uploading/removing torrents when that's not the case.

# License

GNU AGPL v3. See [LICENSE](LICENSE) file.
