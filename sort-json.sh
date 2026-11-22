
cat ./data/script-data.json  | nix run nixpkgs#jq --  'sort_by(.id)' > ./tmp.json

mv ./tmp.json ./data/script-data.json

