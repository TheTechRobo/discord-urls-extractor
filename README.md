# discord-urls-extractor

### Maintainence Level: Updated as I need it.

I'm actually using this program in production! Updates are made when I'm interested, or when they're necessary.

### Information

Created for injesting URLs from DHT scrapes (and now discard2 ones, too!) into the Archiveteam URLs project.

Might not extract all URLs correctly. [#8](https://github.com/TheTechRobo/discordhistorytracker-urls-extractor/pull/8) improved on this, though.

:warning: Every time you run the script, it will overwrite `urls.url` with the URLs from the current script. A good idea is to use a separate directory for each scrape (Cargo's `--manifest-path` can help with that) or back up the urls.url file before you run the script.

Tip: `ignores.url` is a list of URLs that should NOT be extracted. This allows you to run the script on different scrapes without having duplicates. It's also helpful if you have previously used other tools to scrape URLs - just add the URLs you scraped to ignores.url and they won't be scraped again! It's not perfect, but it should work 99% of the time.

- `ignores.url` is read into memory, new URLs are added to it, and then `ignores.url` is overwritten with the loaded values. As such, don't modify `ignores.url` while the script is running - your changes will have no effect, and when the script finishes they will be overwritten!

(Also, I think this goes without saying, but don't run the app multiple times in the same folder. That's a recipe for ~~turnabout~~ disaster.) <!-- fight me -->

## Usage with Discord History Tracker (DESKTOP APP ONLY)
:warning: Note that the DHT extractor is pretty much unmaintained since I no longer use it. I'll fix bugs, but ~~it doesn't support embeds~~ or any new DHT features. It only supports extracting attachment urls (which can now be done via DHT!) and finding URLs in messages using a regex (and getting avatar urls). ~~It does not look through embeds~~ or any other feature.

Once you've got your .dht file, run:

    cargo r <file path> dht

Of course, if the file path has spaces, pad it with quotes (`"`).

Example: `cargo r /home/thetechrobo/Discordbackups/dsicord_data/SteamgridDB/SteamGridDB.dht dht`

## Usage with discard2

:warning: These steps have changed! (The old steps will still work, but the new ones are simpler and get more URLs.)

1. Use the `raw-jsonl` reader from discard2. Save its output into a file. Do not name that file `urls.url` or `ignores.url`.
3. Run `cargo r messages.jsonl discard2`, assuming you named the file in Step 2 `messages.jsonl`.
4. Use a program to get rid of any duplicate URLs. (There shouldn't be any, but I'm not perfect.) On \*nix you can use `sort -u` or `uniq`.

To get even more data (server emojis, role icons, and more), `--parse-websockets`. Note that you then have to specify the `--guild-id` (server ID; can be found in the state.json)

## Usage with plain text
If you have some plain text files, you can use them directly. That will find all URLs saved in the file, or at least most of them. I think.

    cargo r <file path> plaintext
    
## Licence

Licenced under the Apache 2.0 licence. Copyright (C) TheTechRobo, 2021-2022.

>   Copyright 2021-2022 TheTechRobo
>
>   Licensed under the Apache License, Version 2.0 (the "License");
>   you may not use this file except in compliance with the License.
>   You may obtain a copy of the License at
>
>       http://www.apache.org/licenses/LICENSE-2.0
>
>   Unless required by applicable law or agreed to in writing, software
>   distributed under the License is distributed on an "AS IS" BASIS,
>   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
>   See the License for the specific language governing permissions and
>   limitations under the License.
