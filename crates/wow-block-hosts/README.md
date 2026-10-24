# wow-block-hosts

`LD_PRELOAD` library for running the WoW 7.3.5 (26972) client under Wine/Proton. It stops the
client from hanging on the loading screen when entering the world.

## The problem

Under Wine, the client sometimes hangs on the loading screen when entering the world, on any server
core (TrinityCore, LegionCore, AshamaneCore). The client log, `Logs/Client.log`, ends with
`Waiting for active player`. The server log stops after it sends `SMSG_CONNECT_TO`.

Cause, found with a `WINEDEBUG=+winsock` log:

1. At startup the client opens an HTTP connection to `launcher.worldofwarcraft.com:80` and keeps it
   open.
2. When the player clicks Enter World, the client closes that socket. It then connects to the
   server's instance port (8086), when the server's `SMSG_CONNECT_TO` arrives.
3. Wine gives the new socket the same handle number as the closed one.
4. The client's network thread still has its entry for the closed socket. It ties the new socket to
   the old socket's event, so the "connected" event is lost and the client never reads from 8086.

Windows does not reuse a socket handle number that quickly, so the bug does not show there. It is
not a server bug, and the server cannot avoid it: a 1 s delay before `SMSG_CONNECT_TO` was tested on
TrinityCore and still hung, because the client keeps the stale entry however late the new socket is
created.

## Fix 1 (preferred): this library

It makes lookups of the hosts below fail, inside the game process only. The launcher socket is never
opened, so its handle can't be reused. Other programs are not affected.

Blocked by default: `launcher.worldofwarcraft.com`, `blzddist1-a.akamaihd.net`,
`edgecast.blizzard.com`, `level3.blizzard.com`. The last three are the patch CDNs from the
client's `.build.info`; the client contacts them at startup too.

Build:

```sh
cargo build --release -p wow-block-hosts
# -> target/release/libwow_block_hosts.so
```

In Lutris, in the game's Configure → System options → Environment variables, add:

| Key | Value |
|---|---|
| `LD_PRELOAD` | `<azothacore-rs>/target/release/libwow_block_hosts.so` (absolute path) |

`cargo clean` deletes the file; rebuild it after a clean.

Notes:
- The library is 64-bit. Wine's 32-bit helper processes print
  `wrong ELF class: ELFCLASS64 … ignored`. That is harmless, because the game (`Wow-64.exe`) is 64-bit.
- `WOW_BLOCK_HOSTS="a.example b.example"` replaces the list (space- or comma-separated).
- `WOW_BLOCK_HOSTS_DEBUG=1` prints `wow-block-hosts: blocked lookup of <host>` to the game log. Use it
  to check that the library is loaded.
- If something else already sets `LD_PRELOAD` (e.g. gamemode), separate the paths with `:`.

It replaces `getaddrinfo`, `gethostbyname` and `gethostbyname_r`, which Wine uses for Windows name
lookups. Wine has no per-prefix hosts file, so this is the only way to scope the block to one game.

## Fix 2: `/etc/hosts`, without the library

Send the same hosts to `127.0.0.1`, where nothing listens on port 80, so the connections fail at
once. This applies to the **whole system**: no program on this machine can reach these hosts while
the lines are active.

```sh
sudo tee -a /etc/hosts <<'EOF'
# WoW 7.3.5 private server: stop the client keeping Blizzard launcher/CDN sockets open
127.0.0.1 launcher.worldofwarcraft.com
127.0.0.1 blzddist1-a.akamaihd.net
127.0.0.1 edgecast.blizzard.com
127.0.0.1 level3.blizzard.com
EOF
```

To undo, delete those lines (or comment them out with `#`).

Check:

```sh
getent hosts launcher.worldofwarcraft.com   # active: 127.0.0.1 …; undone: a Blizzard address
```

If something listens on port 80 locally (a web server), the client connects to it instead and may
keep that socket open, so the hang can come back. Use Fix 1 in that case.
