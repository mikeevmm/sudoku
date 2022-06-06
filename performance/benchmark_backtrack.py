#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import random
import subprocess
import os

if __name__ == '__main__':
    puzzles = []
    with open('top1465', 'r') as top1465:
        for puzzle in top1465.readlines():
            puzzle = puzzle.strip()
            if not puzzle:
                continue
            puzzle = '\n'.join(' '.join(puzzle.replace('.', '_')[i:i+9]) for i in range(9))
            puzzles.append(puzzle)

    split = 293

    for puzzle in puzzles:
        print(puzzle)
        puzzle_bytes = puzzle.encode('utf-8')
        out = subprocess.run(
                ["../target/release/backtrack",
                    "--benchmark", "-", "-"],
                input=puzzle_bytes, capture_output=True)
        print(out)
        exit(0)
