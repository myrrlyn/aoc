#!/usr/bin/env -S nu --stdin

let session_key = open .env | lines | split column "=" key val | where key == SESSION_ID | get val | first

def "fetch inputs" [year?: int, day?: int] {
	let cal = date now | date to-record | select year day
	let year = if $year == null { $cal | get year } else { $year }
	let day = if $day == null { $cal | get day } else { $day }
	let day_str = $day | fill --alignment right --width 2 --character '0'
	mkdir $"src/y($year)/d($day_str)"
	# print $"assets/inputs/($year)/d($day_str).txt"
	print $"Fetching ($year)/($day_str)"
	http get -H {Cookie: $"session=($session_key)"} $"https://adventofcode.com/($year)/day/($day)/input" | save -f $"src/y($year)/d($day_str)/input.txt"
	sleep 1sec
}

def "gen inputs" [] {
	let today: record<year: int, day: int> = (date now | date to-record | select year day)
	(2015..<$today.year) | each { |y| 1..25 | each { |d| {year: $y, day: $d} } } | flatten | append (1..($today.day) | each { |d| {year: $today.year, day: $d} })
}

# TODO(myrrlyn): figure out how to dispatch to subcommands
def main [year?: int, day?: int] {
	let cal = date now | date to-record | select year day
	let year = if $year == null { $cal | get year } else { $year }
	let day = if $day == null { $cal | get day } else { $day }
	fetch inputs $year $day
}
