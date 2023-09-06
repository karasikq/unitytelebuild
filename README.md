# unitytelebuild

A utility that allows you to build unity projects using a telegram bot, automatically upload to google drive and send a link and build logs to a chat with the bot.

## Features
- Telegram bot for receiving build requests and send results
- Parallel build multiple *different* projects
- Gdrive upload
- White list for users and groups

## What is currentrly supported
- Android builds
- Tested on MacOS\Linux(Fedora 38, Arch)
 - At .env file you may see this 
 ```
 PROJECTS_LOCATION=$HOME/Documents/Projects/Unity
PROJECTS_LOCATION_UNITY='C:/Users/../Documents/Projects'
```
Thats needed for work under Windows with WSL2. First variable for Rust environment, second sends to unity by command argument, but it not tested yet.

## Setup
- Copy Build directory from unityessentials to Assets directory in your Unity project
- Setup Google Cloud project and place keys to client_secrects.json
- Create .env file(or copy example)
 - *DO NOT* specify PROJECTS_LOCATION_UNITY unless you work under WSL
- Setup Telegram bot and paste your token to .env
```
TELOXIDE_BOT_TOKEN='your_token'
```
- Optional fill *config.json* with white-listed user\group ids
- Run with logging enabled 
``` RUST_LOG=Info cargo run ```
## Important
Unity and UnityHub should be installed and available from PATH
