# ZeldaServer

ZeldaServer is a text-based multi-user dungeon (MUD)-style game written in Rust. It provides a foundation for building interactive multiplayer text adventures with a focus on room navigation, player communication, and expandable gameplay systems.

This project builds on the original [Lurk Server](https://github.com/The24Kings/lurk-server) project, adding improvements, refactoring, and additional features to make the codebase more robust and developer-friendly.

> Now with its own dedicated library for the [Lurk Protocol](https://crates.io/crates/lurk_lcsc)!

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
- **Custom protocol** — communication happens over the **Lurk protocol** (not plain telnet)
- **Rust-powered** — built with safety, speed, and concurrency in mind

---

## Requirements

- A **Lurk-compatible client** (telnet or netcat alone will not work)

**To build natively:**

- [Rust](https://www.rust-lang.org/) (latest stable recommended)
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)
- A Unix-like shell (Linux, macOS, or WSL on Windows) to run the `start.sh` script

**To build with Docker:**

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/) (included with Docker Desktop)

---

## Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/The24Kings/ZeldaServer.git
cd ZeldaServer
```

### 2. Configure the environment

Copy the provided `.env` template to `.env.local` and edit it with your desired settings:

```bash
cp .env .env.local
```

The `.env.local` file is required for the server to run.

### 3. Build and start the server

#### Option A: Using the start script

> `start.sh` will automatically build the server in release mode and start an instance running on `8080` with `INFO` verbosity.

```bash
./start.sh [PORT] [VERBOSITY]
```

For example, to run on port `5050` with info debugging:

```bash
./start.sh 5050 -vv
```

#### Option B: Using Docker Compose

Build and start the server with default settings (port `8080`, verbosity `-vv`):

```bash
docker compose up --build
```

To customize the port or verbosity:

```bash
PORT=5050 VERBOSITY=-vvv docker compose up --build
```

To run in the background:

```bash
docker compose up --build -d
```

To stop the server:

```bash
docker compose down
```

Server logs are persisted to the `logs/` directory on the host via a volume mount.

#### Option C: Using Docker directly

Build the image:

```bash
docker build -t zelda-server .
```

Run the container:

```bash
docker run -p 8080:8080 -e PORT=8080 -e VERBOSITY=-vv zelda-server
```

To persist logs, mount the `logs/` directory:

```bash
docker run -p 8080:8080 -e PORT=8080 -e VERBOSITY=-vv -v ./logs:/app/logs zelda-server
```

---

## Playing the Game

ZeldaServer uses the **Lurk protocol**, a custom message-based protocol designed specifically for this project.

To connect and play, you will need a **Lurk-compatible client** that implements the protocol:

- The client is responsible for sending correctly formatted Lurk messages.
- The server will respond with structured Lurk responses (room state, messages, etc.).
- Plain-text clients like `telnet` will not work.

> See the [Lurk Protocol Documentation](https://github.com/The24Kings/LurkProtocol/wiki) for full details on message structure, commands, and expected behavior.

---

## Example Client

If you don’t want to build your own client from scratch, you can try [**LURKMAN**](https://github.com/col1010/LURKMAN), a client that fully implements the Lurk protocol.

This is a great starting point to connect to ZeldaServer and experience the game in action.

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
