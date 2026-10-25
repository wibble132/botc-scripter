
Some of these files are taken from https://github.com/ThePandemoniumInstitute/botc-release/tree/f10cd02e3401af227ce406287eaae7bb99a06a42.

- script-schema.json is located at the repository root
- jinxes.json, nightsheet.json, roles.json are located at resources/data in the botc-release repository.

These files were created by The Pandemonium Institute (TPI), who I am unaffiliated with.
I am using these files under TPI's Community Created Content Policy: https://bloodontheclocktower.com/pages/community-created-content-policy.
This project doesn't compete with TPI's script tool, and is for converting files solely for private use for our Minecraft Server.

# Other files here

### script-data.json

This is a list of the current scripts on the server, with the extra data for minecraft.
Each script has its id, with the json available either at the url specified or given in the script field..

This file is a list of script entries, with each entry being:

{
  "id": "The id of this script",
  "colour": "The Minecraft colour of this script, used for book titles",
  "url": "The URL of the json for this script, defaults to https://scripts.botc.ink/script/<id>.json",
  "script": "The json for this script, overrides the url field"
}