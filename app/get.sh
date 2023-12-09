#!/usr/bin/env bash

# Downloads input

year=${1:-$(date +%Y)}
day=${2:-$(date +%d)}

echo $year / $day

file="assets/inputs/${year}/d$(printf "%02d" ${day}).txt"
# echo $file

# Get the identifier cookie. You must store it in .env at the project root, as
# SESSION_ID=
source .env

curl "https://adventofcode.com/${year}/day/${day}/input" \
  -H "Cookie: session=${SESSION_ID}" \
  --output "${file}"

sleep 1
