# osu-replay-watcher

Due to an issue in danser-go 0.6.9, if ffmpeg is not in path, the replay watcher will not work.

A simple program that watches for new saved osu replays and creates an mp4 file from them using [danser-go](https://github.com/Wieku/danser-go)

This program automatically configures danser-go on the first run, these settings can be changed after in the danser settings file located at `./orw-danser/settings/default.json`.

If you want to change the skin, you can do so in the `./orw-settings/config.json` file.

The videos are created in the `./videos` folder.