#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Run bechmarks on the backtracking solver.

Usage:
    benchmark_backtrack top1465
    benchmark_backtrack paper
    benchmark_backtrack (--help | -h)

Options:
    --help      Show this screen
"""

import random
import subprocess
import os
import numpy as np
import shutil
import docopt
from glob import glob


def main():
    subprocess.run('cargo build --release', shell=True)
    arguments = docopt.docopt(__doc__)

    if arguments['top1465']:
        bench_top1465()
    elif arguments['paper']:
        bench_paper()
    else:
        print('What?')

def bench_top1465():
    puzzles = []
    with open('top1465', 'r') as top1465:
        for puzzle in top1465.readlines():
            puzzle = puzzle.strip()
            if not puzzle:
                continue
            puzzle = '\n'.join(' '.join(puzzle.replace('.', '_')[i*9:(i+1)*9]) for i in range(9))
            puzzles.append(puzzle)

    #split = 293

    if os.path.exists('backtrack.top1465.log'):
        shutil.copy('backtrack.top1465.log', f'backtrack.top1465.log.{random.randint(0, 1000)}.bak')
        os.remove('backtrack.top1465.log')
        
    with open('backtrack.top1465.log', 'w') as outfile:
        outfile.write('# <Puzzle index>\t<Unsolved percentage>\t<Average solve time (ms)>\n')
        for i, puzzle in enumerate(puzzles):
            puzzle_bytes = puzzle.encode('utf-8')
            try:
                out = subprocess.run(
                        ["../target/release/backtrack",
                            "--benchmark", "-", "-"],
                        input=puzzle_bytes,
                        capture_output=True,
                        timeout=90) # Timeout of 1.5 minutes
                print(out.stderr.decode('utf-8'))
                data = np.fromstring(out.stdout, sep='\n')
                unsolved = np.count_nonzero(data < 0.) / data.shape[0]
                solve_time = np.average(data[data >= 0.]) if unsolved < 1. else -1.
            except subprocess.TimeoutExpired:
                unsolved = 1.
                solve_time = -1.
            outfile.write(f'{i}\t{unsolved}\t{solve_time}\n')

def bench_paper():
    if os.path.exists('backtrack.paper.log'):
        shutil.copy('backtrack.paper.log', f'backtrack.paper.log.{random.randint(0, 1000)}.bak')
        os.remove('backtrack.paper.log')
        
    with open('backtrack.paper.log', 'w') as outfile:
        outfile.write('# <Puzzle name>\t<Unsolved fraction>\t<average solve time (ms)>\n')
        for puzzle in glob('paper/*.sudoku'):
            with open(puzzle, 'r') as puzzlebuf:
                puzzle_bytes = puzzlebuf.read().encode('utf-8')
            try:
                out = subprocess.run(
                        ["../target/release/backtrack",
                            "--benchmark", "-", "-"],
                        input=puzzle_bytes,
                        capture_output=True,
                        timeout=90) # Timeout of 1.5 minutes
                print(out.stderr.decode('utf-8'))
                data = np.fromstring(out.stdout, sep='\n')
                print(data)
                unsolved = np.count_nonzero(data < 0.) / data.shape[0]
                solve_time = np.average(data[data >= 0.])
            except subprocess.TimeoutExpired:
                unsolved = 1.
                solve_time = -1.
            outfile.write(f'{os.path.basename(puzzle)}\t{unsolved}\t{solve_time}\n')

if __name__ == '__main__':
    main()
