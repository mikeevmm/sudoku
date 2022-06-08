#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Benchmark the alternated projections technique.

Usage:
    benchmark_projection paper
    benchmark_projection top1465
    benchmark_projection --help

Options:
    --help      Show this screen.
"""

import os
import shutil
import random
import concurrent.futures
from tempfile import NamedTemporaryFile
import numpy as np
from glob import glob
from time import perf_counter
from docopt import docopt
from subprocess import TimeoutExpired, run

def main():
    run('cargo build --release', shell=True)
    arguments = docopt(__doc__)

    if arguments['paper']:
        benchmark_paper()
    elif arguments['top1465']:
        benchmark_top1465()
    else:
        print('what?')


def benchmark_paper():
    if os.path.exists('projection.log'):
        shutil.copy('projection.log',
                    f'projection.log.{random.randint(0, 1000)}.bak')
        os.remove('projection.log')

    with open('projection.log', 'w') as outfile:
        outfile.write(
            '# <Puzzle index>\t<Unsolved percentage>\t<Average solve time (ms)>\n')
        for puzzlefile in glob('paper/*.sudoku'):
            with concurrent.futures.ThreadPoolExecutor(max_workers=8) as executor:
                times = [*executor.map(_thread_benchmark_paper, (puzzlefile for _ in range(4)))]
            times = np.array(times)
            print(times, flush=True)
            outfile.write(
                f'"{puzzlefile}"\t'
                f'{np.count_nonzero(times[times < 0.]) / 4.}\t'
                f'{np.average(times[times >= 0.]) if not (times < 0).all() else 0.}\n')


def _thread_benchmark_paper(puzzlefile):
    try:
        start_time = perf_counter()
        out = run(['../target/release/projection', '100_000_000', puzzlefile],
            timeout=90,
            capture_output=True)
        end_time = perf_counter()
        if out.returncode != 0:
            raise Exception(out.stderr.decode('utf-8'))
            
        stdout = out.stdout.decode('utf-8')
        state = stdout[:stdout.find('\n')].strip()
        final = stdout[stdout.find('\n')+1:].strip()

        if state == 'EXHAUSTED':
            return -1
        elif state == 'ALL SATISFIED':
            return ((end_time - start_time) * 1000)
    except TimeoutExpired:
        return -1
    


def benchmark_top1465():
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

    if os.path.exists('projection.top1465.log'):
        shutil.copy('projection.top1465.log',
                    f'projection.top1465.log.{random.randint(0, 1000)}.bak')
        os.remove('projection.top1465.log')

    with open('projection.top1465.log', 'w') as outfile:
        outfile.write(
            '# <Puzzle index>\t<Unsolved percentage>\t<Average solve time (ms)>\n')
        for i, puzzle in enumerate(puzzles):
            with concurrent.futures.ThreadPoolExecutor(max_workers=8) as executor:
                times = [*executor.map(_thread_benchmark_top1465, (puzzle for _ in range(4)))]
            times = np.array(times)
            print(times, flush=True)
            outfile.write(
                f'{i}\t'
                f'{np.count_nonzero(times[times < 0.]) / 4.}\t'
                f'{np.average(times[times >= 0.]) if not (times < 0).all() else 0.}\n')


def _thread_benchmark_top1465(puzzle):
    with NamedTemporaryFile(delete=True) as puzzlefile:
        puzzlefile.write(puzzle.encode('utf-8'))
        puzzlefile.flush()
        try:
            start_time = perf_counter()
            out = run(['../target/release/projection', '100_000_000', puzzlefile.name],
                timeout=90,
                capture_output=True)
            end_time = perf_counter()
            if out.returncode != 0:
                raise Exception(out.stderr.decode('utf-8'))
                
            stdout = out.stdout.decode('utf-8')
            state = stdout[:stdout.find('\n')].strip()
            final = stdout[stdout.find('\n')+1:].strip()

            if state == 'EXHAUSTED':
                return -1
            elif state == 'ALL SATISFIED':
                return ((end_time - start_time) * 1000)
        except TimeoutExpired:
            return -1


if __name__ == '__main__':
    main()
