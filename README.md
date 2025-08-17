# ImprovedLurk

ImprovedLurk is a text-based multi-user dungeon (MUD)-style game written in Rust. It provides a foundation for building interactive multiplayer text adventures with a focus on room navigation, player communication, and expandable gameplay systems.

This project builds on the original [Lurk Server](https://github.com/The24Kings/lurk-server) project, adding improvements, refactoring, and additional features to make the codebase more robust and developer-friendly.

```
 ______    _     _           _____
|___  /   | |   | |         / ____|
   / / ___| | __| | __ _   | (___   ___ _ ____   _____ _ __
  / / / _ \ |/ _` |/ _` |   \___ \ / _ \ '__\ \ / / _ \ '__|
 / /_|  __/ | (_| | (_| |   ____) |  __/ |   \ V /  __/ |
/_____\___|_|\__,_|\__,_|  |_____/ \___|_|    \_/ \___|_|

You find yourself standing in front of the gaping maw of a towering tree.
You hear a booming voice from above telling you to enter, but beware for danger lay ahead!

         @@@@@@@@@@@@@@@@@@@@@@@@@@@@
      @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
     @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
   @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
  @@@@@@@@@@@@@@  '.@@@@@@@@@@@@@@@@@.--.@@@@@@@@@
    @@@@@@@@\   @@  ¯ @@@@@@@@@@@ '¯¯ ___..@@@@@@
     @@@@@@@@|                 @    .'@@@@@@@@@@
        @@@@@@\                    /@@@@@@@@
               \                  /
               |   .--'|__|'--.   |
               |  /.--'/  \'--.\  |
   __  ___     /      /____\      \     ___
 _(  )(   )_  |     .' .''. '.     |  _(   )__  __      __
(           )_|    |__/    \__|    |_(        )(  )_   (
             /                      \__             )_(¯
_______.---./    .'                    \_.--._ ___________
  --''¯        _/    __                       '--..
             ''    .'
```

---

## Features

- **Room-based world navigation** — players can explore interconnected rooms
- **Player management** — supports multiple players in a shared world
- **Text-based interaction** — send and receive messages in real time
- **Custom protocol** — communication happens over the **LURK protocol** (not plain telnet)
- **Rust-powered** — built with safety, speed, and concurrency in mind

---

## Requirements

- [Rust](https://www.rust-lang.org/) (latest stable recommended)
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)
- A Unix-like shell (Linux, macOS, or WSL on Windows) to run the `start.sh` script
- A **LURK-compatible client** (telnet or netcat alone will not work)

---

## Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/The24Kings/ImprovedLurk.git
cd ImprovedLurk
```

### 2. Build the project

```bash
cargo build --release
```

This produces an optimized binary in `target/release/`.

### 3. Start the server

The recommended way to start the server is with the included script:

```bash
./start.sh [PORT]
```

For example, to run on port `5050`:

```bash
./start.sh 5050
```

---

## Playing the Game

ImprovedLurk uses the **LURK protocol**, a custom message-based protocol designed specifically for this project.

To connect and play, you will need a **LURK-compatible client** that implements the protocol:

- The client is responsible for sending correctly formatted LURK messages.
- The server will respond with structured LURK responses (room state, messages, etc.).
- Plain-text clients like `telnet` will not work.

👉 See the [LURK Protocol Documentation](https://github.com/The24Kings/LurkProtocol/wiki) for full details on message structure, commands, and expected behavior.

---

## Example Client

If you don’t want to build your own client from scratch, you can try [**LURKMAN**](https://github.com/col1010/LURKMAN), a client that fully implements the LURK protocol.

This is a great starting point to connect to an ImprovedLurk server and experience the game in action.

---

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/new-thing`)
3. Commit changes (`git commit -m "Add new thing"`)
4. Push to branch (`git push origin feature/new-thing`)
5. Open a Pull Request

---

## License

This project is licensed under the [MIT License](LICENSE).
