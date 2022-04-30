# Azoth

Discord bot made in Rust

## Currently available commands

- `~ping` replies back with a pong
- `~github <username>` | `~gh <username>`: adds new user to be tracked for github pings, uses the userid of the user of the command to ping them
- `~github_remove` | `~ghrm`: removes the user by discord id from the list of tracked users
- `~quiz <difficulty=easy>`: gets random question from the open trivia database, can pass in a difficutly, either `easy`, `medium`, or `hard`. will by default fetch easy questions
- `~scores`: displays the scores of everyone who has used the bot

## Done

- [x] Run the code in the `github` command on a timer and auto send message based on result
  - [x] Can add or remove accounts to check
  - [x] Need to customize message based on response from github
  - [x] Saves users to checks somewhere so that the bot does not reset
  - [x] Loads data so that the bot does not reset
- [x] `quiz` gets 1 question from the trivia API and person who requested the original question can answer
  - [x] as prototype, get a question from the database
    - [x] questions can be answered as a react
    - [x] questions can be answered from dropdown (need to check discord api to see how that is doable, example from Aether Hunts discord server)
    - [x] handles true/false and multiple choice questions
 
## Potential Future Additions
- [ ] Recurring Changes
  - [ ] Is able to give a schedule of when to ping instead of it being duration based (i.e. a [cron schedule](https://github.com/mehcode/schedule-rs))
- [ ] Quiz Changes
  - [ ] need to fix the output with special characters (like quotes) in discord message
  - [ ] create a full game
    - [ ] custom player length
    - [ ] keep track of players score during game
    - [ ] keep track of whose turn it is, random people cant interfere with answers
  - [ ] keep single quiz questions but make them random difficulty
