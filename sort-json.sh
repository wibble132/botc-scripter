
cat ./data/script-data.json  | jq 'sort_by(.id)' > ./tmp.json

mv ./tmp.json ./data/script-data.json

