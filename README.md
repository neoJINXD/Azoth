# Azoth

Discord bot made in Rust

## Currently available commands

- `~ping` replies back with a pong
- `~github <username>` | `~gh <username>`: adds new user to be tracked for github pings, uses the userid of the user of the command to ping them
- `~github_remove` | `~ghrm`: removes the user by discord id from the list of tracked users
- `~quiz`: gets random question from the open trivia database

## Still needs implementing

- [x] Run the code in the `github` command on a timer and auto send message based on result
  - [x] Can add or remove accounts to check
  - [x] Need to customize message based on response from github
  - [x] Saves users to checks somewhere so that the bot does not reset
  - [x] Loads data so that the bot does not reset
- [ ] `quiz` gets 1 question from the trivia API and person who requested the original question can answer with a reply
  - [x] as prototype, get a question from the database
    - [x] questions can be answered as a react
    - [x] questions can be answered from dropdown (need to check discord api to see how that is doable, example from Aether Hunts discord server)
    - [x] handles true/false and multiple choice questions
  - [ ] need to fix the output with special characters (like quotes) in discord message
  - [ ] create game from original message
    - [ ] custom player length
    - [ ] keep track of players score
    - [ ] keep track of whose turn it is, random people cant interfere
