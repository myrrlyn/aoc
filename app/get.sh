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
	-H 'Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8' \
	-H 'Accept-Language: en-US,en;q=0.5' \
	-H 'DNT: 1' \
	-H 'Connection: keep-alive' \
	-H 'Upgrade-Insecure-Requests: 1' \
	-H 'Sec-Fetch-Dest: document' \
	-H 'Sec-Fetch-Mode: navigate' \
	-H 'Sec-Fetch-Site: same-origin' \
	-H 'Sec-GPC: 1' \
	-H 'Pragma: no-cache' \
	-H 'Cache-Control: no-cache' \
	-H 'TE: trailers' \
	-H "Cookie: session=${SESSION_ID}" \
	--output "${file}"

sleep 1
