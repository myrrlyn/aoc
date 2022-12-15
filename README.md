# myrrlyn Advent of Code

This is where I store all my Advent of Code work.

## Structure

- `years/`: Each year gets a folder in here
  - `<a year>/`
    - `d{01..25}/`: Every day gets its own folder
      - `<a language>/`: Each language used to solve the day gets *its* own
        folder.
      - `input.txt`: Puzzle input, mandatory
      - `sample.txt`: Sample input, optional
- `<a language>/`: Each language used to solve *any* puzzle gets its own
  top-level folder used as a common library across the entire project. This is
  supposed to enable code re-use between puzzles.

Packages will be named in the form `wyz_aoc_<year>_d<day>`.
