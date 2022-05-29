#!/bin/bash

set -e

# Run the annealing once.
FIRST=("$(cargo run --release --bin annealing -- example.sudoku remelt.schedule 2>/dev/null)")

STATUS=("$(echo "$FIRST" | head -n1)")
HINT=("$(echo "$FIRST" | tail +2)")

if [ $STATUS == "GLASS" ]; then
    echo "Glassy state, reannealing..."
    SECOND=("$(echo "$HINT" | cargo run --release --bin annealing -- example.sudoku reanneal.schedule -)")

    STATUS=("$(echo "$SECOND" | head -n1)")
    HINT=("$(echo "$SECOND" | tail +2)")

    echo "$SECOND"
else
    echo "$FIRST"
fi