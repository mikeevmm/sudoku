#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Benchmark the annealing technique.

Usage:
    benchmark_annealing paper [--remelt]
    benchmark_annealing top1465 [--remelt]
    benchmark_annealing (--help|-h)

Options:
    --remelt    Use the remelt schedule, rather than the simple geometric
                schedule.
    --help      Show this screen.
"""

import os
import shutil
import time
import copy
import numpy as np
from tempfile import NamedTemporaryFile
from subprocess import run
from docopt import docopt


def main():
    arguments = docopt(__doc__)

    run('cargo build --release', shell=True)

    if arguments['paper']:
        benchmark_paper()
    elif arguments['top1465']:
        if arguments['--remelt']:
            pass
        else:
            benchmark_top1465_geometric()
    else:
        print('What?')


def benchmark_paper():
    pass


def benchmark_top1465_geometric():
    puzzles = []
    with open('top1465', 'r') as top1465:
        for puzzle in top1465.readlines():
            puzzle = puzzle.strip()
            if not puzzle:
                continue
            puzzle = '\n'.join(' '.join(puzzle.replace('.', '_')[i:i+9]) for i in range(9))
            puzzles.append(puzzle)
    
    print('Finished parsing puzzles.')

    if os.path.exists('annealing.top1465.log'):
        shutil.copy('annealing.top1465.log', f'annealing.top1465.log.{random.randint(0, 1000)}.bak')
        os.remove('annealing.top1465.log')

    with open('backtrack.top1465.log', 'w') as outfile:
        outfile.write('# <Puzzle index>\t<Unsolved percentage>\t<Average solve time (ms)>\n')
        for _i, puzzle in enumerate(puzzles):
            with NamedTemporaryFile() as puzzlefile, NamedTemporaryFile() as hintfile:
                puzzlefile.write(puzzle.encode('utf-8'))
                puzzlefile.flush()

                # Give ourselves 1m30s.
                # Use a geometric schedule with 100_000 iterations total each round
                # Start with a starting temperature of 1.,
                # then every new round halve scale the curve by 1/2.
                start_time = time.perf_counter()
                start_temp = 1.
                while True:
                    if time.perf_counter() - start_time > 90 * 1000:
                        break

                    schedule = ''
                    total_iters = 1
                    temperature = copy.copy(start_temp)

                    while total_iters < 100_000:
                        schedule += f'{temperature} {int(total_iters)}\n'
                        temperature *= 0.99
                        total_iters *= 1.01

                    result = run(
                        ['../target/release/annealing', puzzlefile.name, '-', hintfile.name],
                        input=schedule.encode('utf-8'),
                        timeout=(start_time + 90 * 1000 - time.perf_counter()),
                        capture_output=True)

                    if result.returncode != 0:
                        raise Exception(result.stderr.decode('utf-8'))

                    output = result.stdout.decode('utf-8')
                    state = output[:output.find('\n')].strip()
                    final = output[output.find('\n')+1:].strip()

                    if state == 'GLASS':
                        start_temp *= .5
                        hintfile.truncate()
                        hintfile.write(final.encode('utf-8'))
                        hintfile.flush()
                        continue # Reanneal
                    elif state == 'SUCCESS':
                        # TODO
                        break

if __name__ == '__main__':
    main()
