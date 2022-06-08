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
import random
import numpy as np
from glob import glob
from tempfile import NamedTemporaryFile
from subprocess import run, TimeoutExpired
from docopt import docopt


def main():
    arguments = docopt(__doc__)

    run('cargo build --release', shell=True)

    if arguments['paper']:
        if arguments['--remelt']:
            benchmark_paper_remelt()
        else:
            benchmark_paper_geometric()
    elif arguments['top1465']:
        if arguments['--remelt']:
            benchmark_top1465_remelt()
        else:
            benchmark_top1465_geometric()
    else:
        print('What?')


def benchmark_paper_remelt():
    if os.path.exists('annealing.remelt.paper.log'):
        shutil.copy('annealing.remelt.paper.log',
                    f'annealing.remelt.paper.log.{random.randint(0, 1000)}.bak')
        os.remove('annealing.remelt.paper.log')

    orig_schedule = np.loadtxt('remelt.schedule', delimiter='\t')

    with open('annealing.remelt.paper.log', 'w') as outfile:
        outfile.write(
            '# <Puzzle index>\t<Unsolved percentage>\t<Average solve time (ms)>\n')
        for i, puzzlefile in enumerate(glob('paper/*.sudoku')):
            times = np.array([-1., -1., -1., -1.])
            for iteration in range(4):
                with NamedTemporaryFile(delete=True) as hintfile:
                    # Give ourselves 1m30s.
                    # Use a geometric schedule with 100_000 iterations total each round
                    # Start with a starting temperature of 1.,
                    # then every new round halve scale the curve by 1/2.
                    start_time = time.perf_counter()
                    start_temp = 1.
                    args = []
                    while True:
                        if time.perf_counter() - start_time > 90:
                            break

                        if start_temp < 0.01:
                            start_temp = 1.
                        else:
                            start_temp *= 0.99

                        schedule = orig_schedule.copy()
                        schedule[:,0] *= start_temp
                        schedule = '\n'.join('%.6g\t%d' % (line[0], line[1]) for line in schedule)

                        try:
                            result = run(
                                ['../target/release/annealing', puzzlefile, '-', *args],
                                input=schedule.encode('utf-8'),
                                timeout=(start_time + 90 - time.perf_counter()),
                                capture_output=True)
                        except TimeoutExpired:
                            break

                        if result.returncode != 0:
                            raise Exception(result.stdout.decode('utf-8') +
                                            '\n' + result.stderr.decode('utf-8'))

                        output = result.stdout.decode('utf-8')
                        state = output[:output.find('\n')].strip()
                        final = output[output.find('\n')+1:].strip()

                        if state == 'GLASS':
                            start_temp *= .5
                            hintfile.seek(0)
                            hintfile.truncate()
                            hintfile.write(final.encode('utf-8'))
                            hintfile.flush()
                            args = [hintfile.name]
                            continue  # Reanneal
                        elif state == 'SUCCESS':
                            times[iteration] = (time.perf_counter() - start_time) * 1000
                            break

            print(times)
            outfile.write(
                f'{i}\t'
                f'{np.count_nonzero(times[times < 0.]) / 4.}\t'
                f'{np.average(times[times >= 0.]) if not (times < 0).all() else 0.}\n')


def benchmark_paper_geometric():
    if os.path.exists('annealing.paper.log'):
        shutil.copy('annealing.paper.log',
                    f'annealing.paper.log.{random.randint(0, 1000)}.bak')
        os.remove('annealing.paper.log')

    with open('annealing.paper.log', 'w') as outfile:
        outfile.write(
            '# <Puzzle index>\t<Unsolved percentage>\t<Average solve time (ms)>\n')
        for i, puzzlefile in enumerate(glob('paper/*.sudoku')):
            times = np.array([-1., -1., -1., -1.])
            for iteration in range(4):
                with NamedTemporaryFile(delete=True) as hintfile:
                    # Give ourselves 1m30s.
                    # Use a geometric schedule with 100_000 iterations total each round
                    # Start with a starting temperature of 1.,
                    # then every new round halve scale the curve by 1/2.
                    start_time = time.perf_counter()
                    start_temp = 1.
                    args = []
                    while True:
                        if time.perf_counter() - start_time > 90:
                            break

                        if start_temp < 0.01:
                            start_temp = 1.

                        schedule = ''
                        temperature = copy.copy(start_temp)
                        total_iters = 0
                        iterations = 1
                        while total_iters < 100_000:
                            schedule += f'{temperature} {int(iterations)}\n'
                            temperature *= 0.99
                            iterations *= 1.01
                            total_iters += iterations

                        try:
                            result = run(
                                ['../target/release/annealing', puzzlefile, '-', *args],
                                input=schedule.encode('utf-8'),
                                timeout=(start_time + 90 - time.perf_counter()),
                                capture_output=True)
                        except TimeoutExpired:
                            break

                        if result.returncode != 0:
                            raise Exception(result.stdout.decode('utf-8') +
                                            '\n' + result.stderr.decode('utf-8'))

                        output = result.stdout.decode('utf-8')
                        state = output[:output.find('\n')].strip()
                        final = output[output.find('\n')+1:].strip()

                        if state == 'GLASS':
                            start_temp *= .5
                            hintfile.seek(0)
                            hintfile.truncate()
                            hintfile.write(final.encode('utf-8'))
                            hintfile.flush()
                            args = [hintfile.name]
                            continue  # Reanneal
                        elif state == 'SUCCESS':
                            times[iteration] = (time.perf_counter() - start_time) * 1000
                            break

            print(times)
            outfile.write(
                f'{i}\t'
                f'{np.count_nonzero(times[times < 0.]) / 4.}\t'
                f'{np.average(times[times >= 0.])}\n')


def benchmark_top1465_geometric():
    puzzles = []
    with open('top1465', 'r') as top1465:
        for puzzle in top1465.readlines():
            puzzle = puzzle.strip()
            if not puzzle:
                continue
            puzzle = '\n'.join(' '.join(puzzle.replace(
                '.', '_')[i:i+9]) for i in range(9))
            puzzles.append(puzzle)

    print('Finished parsing puzzles.')

    if os.path.exists('annealing.remelt.top1465.log'):
        shutil.copy('annealing.remelt.top1465.log',
                    f'annealing.remelt.top1465.log.{random.randint(0, 1000)}.bak')
        os.remove('annealing.remelt.top1465.log')

    with open('annealing.remelt.top1465.log', 'w') as outfile:
        outfile.write(
            '# <Puzzle index>\t<Unsolved percentage>\t<Average solve time (ms)>\n')
        for i, puzzle in enumerate(puzzles):
            times = np.array([-1., -1., -1., -1.])
            for iteration in range(4):
                with NamedTemporaryFile(delete=True) as puzzlefile, \
                        NamedTemporaryFile(delete=True) as hintfile:
                    puzzlefile.write(puzzle.encode('utf-8'))
                    puzzlefile.flush()

                    # Give ourselves 1m30s.
                    # Use a geometric schedule with 100_000 iterations total each round
                    # Start with a starting temperature of 1.,
                    # then every new round halve scale the curve by 1/2.
                    start_time = time.perf_counter()
                    start_temp = 1.
                    args = []
                    while True:
                        if time.perf_counter() - start_time > 90:
                            break

                        if start_temp < 0.01:
                            start_temp = 1.

                        schedule = ''
                        temperature = copy.copy(start_temp)
                        iterations = 1
                        total_iters = 0
                        while total_iters < 100_000:
                            schedule += f'{temperature} {int(iterations)}\n'
                            temperature *= 0.99
                            iterations *= 1.01
                            total_iters += iterations

                        try:
                            result = run(
                                ['../target/release/annealing',
                                    puzzlefile.name, '-', *args],
                                input=schedule.encode('utf-8'),
                                timeout=(start_time + 90 - time.perf_counter()),
                                capture_output=True)
                        except TimeoutExpired:
                            break
                            
                        if result.returncode != 0:
                            raise Exception(result.stdout.decode('utf-8') +
                                            '\n' + result.stderr.decode('utf-8'))

                        output = result.stdout.decode('utf-8')
                        state = output[:output.find('\n')].strip()
                        final = output[output.find('\n')+1:].strip()

                        if state == 'GLASS':
                            start_temp *= .5
                            hintfile.seek(0)
                            hintfile.truncate()
                            hintfile.write(final.encode('utf-8'))
                            hintfile.flush()
                            args = [hintfile.name]
                            continue  # Reanneal
                        elif state == 'SUCCESS':
                            times[iteration] = (time.perf_counter() - start_time) * 1000
                            break

            print(times)
            outfile.write(
                f'{i}\t'
                f'{np.count_nonzero(times[times < 0.]) / 4.}\t'
                f'{np.average(times[times >= 0.])}\n')

def benchmark_top1465_remelt():
    puzzles = []
    with open('top1465', 'r') as top1465:
        for puzzle in top1465.readlines():
            puzzle = puzzle.strip()
            if not puzzle:
                continue
            puzzle = '\n'.join(' '.join(puzzle.replace(
                '.', '_')[i:i+9]) for i in range(9))
            puzzles.append(puzzle)

    print('Finished parsing puzzles.')

    if os.path.exists('annealing.remelt.top1465.log'):
        shutil.copy('annealing.remelt.top1465.log',
                    f'annealing.remelt.top1465.log.{random.randint(0, 1000)}.bak')
        os.remove('annealing.remelt.top1465.log')

    orig_schedule = np.loadtxt('remelt.schedule', delimiter='\t')

    with open('annealing.remelt.top1465.log', 'w') as outfile:
        outfile.write(
            '# <Puzzle index>\t<Unsolved percentage>\t<Average solve time (ms)>\n')
        for i, puzzle in enumerate(puzzles):
            times = np.array([-1., -1., -1., -1.])
            for iteration in range(4):
                with NamedTemporaryFile(delete=True) as puzzlefile, \
                        NamedTemporaryFile(delete=True) as hintfile:
                    puzzlefile.write(puzzle.encode('utf-8'))
                    puzzlefile.flush()

                    start_time = time.perf_counter()
                    start_temp = 1.
                    args = []
                    while True:
                        if time.perf_counter() - start_time > 90:
                            break

                        if start_temp < 0.01:
                            start_temp = 1.
                        else:
                            start_temp *= 0.99

                        schedule = orig_schedule.copy()
                        schedule[:,0] *= start_temp
                        schedule = '\n'.join('%.6g\t%d' % (line[0], line[1]) for line in schedule)

                        try:
                            result = run(
                                ['../target/release/annealing', puzzlefile.name, '-', *args],
                                input=schedule.encode('utf-8'),
                                timeout=(start_time + 90 - time.perf_counter()),
                                capture_output=True)
                        except TimeoutExpired:
                            break

                        if result.returncode != 0:
                            raise Exception(result.stdout.decode('utf-8') +
                                            '\n' + result.stderr.decode('utf-8'))

                        output = result.stdout.decode('utf-8')
                        state = output[:output.find('\n')].strip()
                        final = output[output.find('\n')+1:].strip()

                        if state == 'GLASS':
                            start_temp *= .5
                            hintfile.seek(0)
                            hintfile.truncate()
                            hintfile.write(final.encode('utf-8'))
                            hintfile.flush()
                            args = [hintfile.name]
                            continue  # Reanneal
                        elif state == 'SUCCESS':
                            times[iteration] = (time.perf_counter() - start_time) * 1000
                            break

            print(times)
            outfile.write(
                f'{i}\t'
                f'{np.count_nonzero(times[times < 0.]) / 4.}\t'
                f'{np.average(times[times >= 0.]) if not (times < 0).all() else 0.}\n')



if __name__ == '__main__':
    main()
