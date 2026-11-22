#!/usr/bin/env bash

SCRIPT_ID=$1

if [ -z "$SCRIPT_ID" ]; then
  echo "Missing script ID argument"
  echo "Usage: $0 <SCRIPT ID>"
  exit 1
fi

SCRIPT_FILE="./out/script/$SCRIPT_ID.mcfunction"
if [ ! -f "$SCRIPT_FILE" ]; then
  echo "Could not find script file at $SCRIPT_FILE"
  exit 1
fi

ROLES_FILE="./out/roles/$SCRIPT_ID.mcfunction"
if [ ! -f "$ROLES_FILE" ]; then
  echo "Could not find roles file at $ROLES_FILE"
  exit 1
fi

echo "Uploading $SCRIPT_ID to server..."

scp "$SCRIPT_FILE" "mcservermain:/world/datapacks/novelwit-botc-lite/data/botc_nw_lite/function/script/$SCRIPT_ID.mcfunction"
scp "$ROLES_FILE" "mcservermain:/world/datapacks/novelwit-botc-lite/data/botc_nw_lite/function/roles/$SCRIPT_ID.mcfunction"

