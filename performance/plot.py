#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Plot all the things!

Usage:
    plot backtrack (paper|top1465)
    plot annealing schedules
    plot annealing (geometric|remelt) (paper|top1465)
    plot projection (paper|top1465)
    plot (--help|-h)

Options:
    --help      Show this screen
"""

from shutil import which
import matplotlib.pyplot as plt
import numpy as np
from docopt import docopt


def plot_backtrack_paper():
    data = np.loadtxt('backtrack.paper.log', delimiter='\t',
                      dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                             'formats': ('S25', 'f4', 'f4')})

    assert (data['frac_failed'] == 0.).all()

    sort_key = np.argsort(-data['ms_time_avg'])
    x = np.arange(data['puzzle'].shape[0])

    p95 = data['ms_time_avg'][sort_key[int(.05 * sort_key.shape[0]):]]
    p50 = data['ms_time_avg'][sort_key[int(.50 * sort_key.shape[0]):]]

    plt.scatter(x, data['ms_time_avg'][sort_key], marker='1')

    plt.axhline(np.average(data['ms_time_avg']),
                color='#D23D3D', label='average', linestyle='--')
    plt.axhline(np.average(p95), color='#3ED53D',
                label='P95 average', linestyle='--')
    plt.axhline(np.average(p50), color='#3C3DD5',
                label='P50 average', linestyle='--')

    plt.xticks([])
    plt.ylabel('Time (ms, average over 4 runs)')
    plt.legend(loc='center right')
    plt.savefig('backtrack_paper.png')


def plot_backtrack_top1465():
    data = np.loadtxt('backtrack.top1465.log', delimiter='\t',
                      dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                             'formats': ('i1', 'f4', 'f4')})
    data['ms_time_avg'] /= 1000  # Convert to seconds

    x = np.arange(data['puzzle'].shape[0])

    non_failed = data['ms_time_avg'][data['frac_failed'] < 1.]
    non_failed_key = np.argsort(-non_failed)
    non_failed_x = np.linspace(0, len(x), len(non_failed))

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(
        8, 4), gridspec_kw={'width_ratios': [8, 3]})
    fig.subplots_adjust(wspace=0)

    ax1.scatter(non_failed_x, non_failed[non_failed_key], s=.5, marker='1')
    ax1.set_ylabel('Run time (seconds, average over 4 iterations)')
    ax1.set_xticks([])

    top50 = non_failed[non_failed_key[:50]]
    p95 = non_failed[non_failed_key[non_failed_key[int(.05 * non_failed_key.shape[0]):]]]

    ax1.axhline(np.average(non_failed), color='#D23D3D',
                label='average ({:.3f}s)'.format(np.average(non_failed)), linestyle='--')
    ax1.axhline(np.average(p95), color='#3ED53D', label='P95 average ({:.3f}s)'.format(
        np.average(p95)), linestyle='--')
    ax1.axhline(np.average(top50), color='#3050CD', label='top 50 average ({:.3f}s)'.format(
        np.average(top50)), linestyle='--')

    ax1.legend(loc='upper right')

    total_frac_failed = np.count_nonzero(
        data['frac_failed'] == 1.0) / len(data['frac_failed'])
    ax2.bar(0, 1. - total_frac_failed, width=1., label='succeeded')
    ax2.bar(0, total_frac_failed, bottom=(
        1. - total_frac_failed), width=1., label='failed')
    ax2.set_xlim(-3., 3.)
    ax2.legend(loc='center right')

    ax2.axis('off')

    plt.savefig('backtrack_top1465.png')


def plot_schedules():
    # Plot geometric
    fig, ax1 = plt.subplots(figsize=(10, 8))
    ax2 = ax1.twinx()

    ax1.set_ylabel('Reduced Temperature', color='#C23D3E')
    ax2.set_ylabel('Iterations', color='#3DC23E')

    temperatures = []
    iterations = []

    iteration = 1.
    total_iters = 0
    temperature = 1.
    while int(total_iters) < 100000:
        iterations.append(int(iteration))
        temperatures.append(temperature)

        temperature *= 0.99
        iteration *= 1.01
        total_iters += int(iteration)

    ax1.scatter(np.arange(len(temperatures)),
                temperatures, color='#C23D3E', marker='1')
    ax2.scatter(np.arange(len(iterations)), iterations,
                color='#3DC23E', marker='2')

    plt.title('Geometric anenaling schedule')
    plt.xticks([])
    plt.savefig('geometric_annealing.png')

    # Plot remelt
    fig, ax1 = plt.subplots(figsize=(10, 8))
    ax2 = ax1.twinx()

    ax1.set_ylabel('Reduced Temperature', color='#C23D3E')
    ax2.set_ylabel('Iterations', color='#3DC23E')

    schedule = np.loadtxt('remelt.schedule', delimiter='\t')

    ax1.scatter(np.arange(len(schedule[:, 0])),
                schedule[:, 0], color='#C23D3E', marker='1')
    ax2.scatter(np.arange(len(schedule[:, 1])),
                schedule[:, 1], color='#3DC23E', marker='2')

    plt.title('“Remelt” anenaling schedule')
    plt.xticks([])
    plt.savefig('remelt_annealing.png')


def plot_geometric_paper():
    data = np.loadtxt('annealing.paper.log', delimiter='\t',
                      dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                             'formats': ('i1', 'f4', 'f4')})
    data['ms_time_avg'] /= 1000  # Convert to seconds

    fig, ax1 = plt.subplots(figsize=(12, 6))
    ax2 = ax1.twinx()

    ax1.tick_params(axis='y', color='#AC3230')
    ax2.tick_params(axis='y', color='#3032AC')
    ax2.set_ylim(-0.05,1.05)

    ax1.tick_params(axis='x', which='both', bottom=False, top=False, labelbottom=False)

    ax1.set_ylabel('Solve time (seconds, average over at most 4 runs)', color='#AC3230')
    ax2.set_ylabel('Fraction solved (out of 4)', color='#3032AC')

    sort_key = np.lexsort((-data['ms_time_avg'], data['frac_failed']))

    x_axis_len = len(data['puzzle'])

    success_x = np.linspace(0, x_axis_len, np.count_nonzero(data['frac_failed'] < 1.))
    success = data[sort_key[data[sort_key]['frac_failed'] < 1.]]['ms_time_avg']
    
    ax1.scatter(success_x, success, marker='1', color='#AC3230')

    ax2.scatter(np.arange(x_axis_len), 1. - data[sort_key]['frac_failed'], color='#3032AC')

    plt.savefig('paper_geometric_annealing.png')


def plot_geometric_top1465():
    data = np.loadtxt('annealing.top1465.log', delimiter='\t',
                      dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                             'formats': ('i1', 'f4', 'f4')})
    data['ms_time_avg'] /= 1000  # Convert to seconds

    fig, ax1 = plt.subplots(figsize=(12, 6))
    ax2 = ax1.twinx()

    ax1.tick_params(axis='y', color='#AC3230')
    ax2.tick_params(axis='y', color='#3032AC')
    ax2.set_ylim(-0.05,1.05)

    ax1.tick_params(axis='x', which='both', bottom=False, top=False, labelbottom=False)

    ax1.set_ylabel('Solve time (seconds, average over at most 4 runs)', color='#AC3230')
    ax2.set_ylabel('Fraction solved (out of 4)', color='#3032AC')

    sort_key = np.lexsort((-data['ms_time_avg'], data['frac_failed']))

    x_axis_len = len(data['puzzle'])

    success_x = np.arange(0, np.count_nonzero(data['frac_failed'] < 1.))
    success = data[sort_key[data[sort_key]['frac_failed'] < 1.]]['ms_time_avg']
    
    ax1.scatter(success_x, success, marker='1', color='#AC3230', s=.08)

    ax2.scatter(np.arange(x_axis_len), 1. - data[sort_key]['frac_failed'], s=.1, color='#3032AC')

    plt.savefig('top1465_geometric_annealing.png')


def plot_remelt_paper():
    data = np.loadtxt('annealing.remelt.log', delimiter='\t',
                      dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                             'formats': ('i1', 'f4', 'f4')})
    data['ms_time_avg'] /= 1000  # Convert to seconds

    compare_data = np.loadtxt('annealing.paper.log', delimiter='\t',
                      dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                             'formats': ('i1', 'f4', 'f4')})
    compare_data['ms_time_avg'] /= 1000  # Convert to seconds

    fig, ax1 = plt.subplots(figsize=(12, 6))
    ax2 = ax1.twinx()

    ax1.tick_params(axis='y', color='#AC3230')
    ax2.tick_params(axis='y', color='#3032AC')
    ax2.set_ylim(-0.05,1.05)

    ax1.tick_params(axis='x', which='both', bottom=False, top=False, labelbottom=False)

    ax1.set_ylabel('Solve time (seconds, average over at most 4 runs)', color='#AC3230')
    ax2.set_ylabel('Fraction solved (out of 4)', color='#3032AC')

    sort_key = np.lexsort((-data['ms_time_avg'], data['frac_failed']))
    compare_sort_key = np.lexsort((-compare_data['ms_time_avg'], compare_data['frac_failed']))

    x_axis_len = len(data['puzzle'])

    success_x = np.arange(0, np.count_nonzero(data['frac_failed'] < 1.))
    success = data[sort_key[data[sort_key]['frac_failed'] < 1.]]['ms_time_avg']
    
    ax1.scatter(success_x, success, marker='1', color='#AC3230')

    compare_success_x = np.arange(0, np.count_nonzero(compare_data['frac_failed'] < 1.))
    compare_success = compare_data[compare_sort_key[compare_data[compare_sort_key]['frac_failed'] < 1.]]['ms_time_avg']
    
    ax1.scatter(success_x, success, marker='1', color='#AC3230')
    ax1.scatter(compare_success_x, compare_success, marker='2', color='#32AC30', alpha=.6, label='Geometric schedule')

    ax2.scatter(np.arange(x_axis_len), 1. - data[sort_key]['frac_failed'], color='#3032AC')

    ax1.legend(loc='center right')

    plt.savefig('paper_remelt_annealing.png')



def plot_remelt_top1465():
    data = np.loadtxt('annealing.remelt.top1465.log', delimiter='\t',
                      dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                             'formats': ('i1', 'f4', 'f4')})
    data['ms_time_avg'] /= 1000  # Convert to seconds

    compare_data = np.loadtxt('annealing.top1465.log', delimiter='\t',
                      dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                             'formats': ('i1', 'f4', 'f4')})
    compare_data['ms_time_avg'] /= 1000  # Convert to seconds


    fig, ax1 = plt.subplots(figsize=(12, 6))
    ax2 = ax1.twinx()

    ax1.tick_params(axis='y', color='#AC3230')
    ax2.tick_params(axis='y', color='#3032AC')
    ax2.set_ylim(-0.05,1.05)

    ax1.tick_params(axis='x', which='both', bottom=False, top=False, labelbottom=False)

    ax1.set_ylabel('Solve time (seconds, average over at most 4 runs)', color='#AC3230')
    ax2.set_ylabel('Fraction solved (out of 4)', color='#3032AC')

    sort_key = np.lexsort((-data['ms_time_avg'], data['frac_failed']))
    compare_sort_key = np.lexsort((-compare_data['ms_time_avg'], compare_data['frac_failed']))

    x_axis_len = len(data['puzzle'])

    success_x = np.arange(0, np.count_nonzero(data['frac_failed'] < 1.))
    success = data[sort_key[data[sort_key]['frac_failed'] < 1.]]['ms_time_avg']

    compare_success_x = np.arange(0, np.count_nonzero(compare_data['frac_failed'] < 1.))
    compare_success = compare_data[compare_sort_key[compare_data[compare_sort_key]['frac_failed'] < 1.]]['ms_time_avg']
    
    ax1.scatter(success_x, success, marker='1', color='#AC3230', s=.08)
    ax1.scatter(compare_success_x, compare_success, marker='2', color='#32AC30', s=.08, alpha=.4, label='Geometric schedule')

    ax2.scatter(np.arange(x_axis_len), 1. - data[sort_key]['frac_failed'], s=.1, color='#3032AC')
    ax2.scatter(np.arange(x_axis_len), 1. - compare_data[compare_sort_key]['frac_failed'], s=.1, alpha=.4, color='#30AC32')

    lgnd = ax1.legend(loc='upper right')
    lgnd.legendHandles[0]._sizes = [30]

    plt.savefig('top1465_remelt_annealing.png')


def plot_projection_paper():
    data = np.loadtxt('projection.log', delimiter='\t',
                      dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                             'formats': ('i1', 'f4', 'f4')})
    data['ms_time_avg'] /= 1000  # Convert to seconds

    fig, ax1 = plt.subplots(figsize=(12, 6))
    ax2 = ax1.twinx()

    ax1.tick_params(axis='y', color='#AC3230')
    ax2.tick_params(axis='y', color='#3032AC')
    ax2.set_ylim(-0.05,1.05)

    ax1.tick_params(axis='x', which='both', bottom=False, top=False, labelbottom=False)

    ax1.set_ylabel('Solve time (seconds, average over at most 4 runs)', color='#AC3230')
    ax2.set_ylabel('Fraction solved (out of 4)', color='#3032AC')

    sort_key = np.lexsort((-data['ms_time_avg'], data['frac_failed']))

    x_axis_len = len(data['puzzle'])

    success_x = np.arange(0, np.count_nonzero(data['frac_failed'] < 1.))
    success = data[sort_key[data[sort_key]['frac_failed'] < 1.]]['ms_time_avg']

    ax1.scatter(success_x, success, marker='1', color='#AC3230', s=.08)
    ax2.scatter(np.arange(x_axis_len), 1. - data[sort_key]['frac_failed'], s=.1, color='#3032AC')

    plt.savefig('paper_projection.png')


def plot_projection_top1465():
    data = np.loadtxt('projection.top1465.log', delimiter='\t',
                      dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                             'formats': ('i1', 'f4', 'f4')})
    data['ms_time_avg'] /= 1000  # Convert to seconds

    fig, ax1 = plt.subplots(figsize=(12, 6))
    ax2 = ax1.twinx()

    ax1.tick_params(axis='y', color='#AC3230')
    ax2.tick_params(axis='y', color='#3032AC')
    ax2.set_ylim(-0.05,1.05)

    ax1.tick_params(axis='x', which='both', bottom=False, top=False, labelbottom=False)

    ax1.set_ylabel('Solve time (seconds, average over at most 4 runs)', color='#AC3230')
    ax2.set_ylabel('Fraction solved (out of 4)', color='#3032AC')

    sort_key = np.lexsort((-data['ms_time_avg'], data['frac_failed']))

    x_axis_len = len(data['puzzle'])

    success_x = np.arange(0, np.count_nonzero(data['frac_failed'] < 1.))
    success = data[sort_key[data[sort_key]['frac_failed'] < 1.]]['ms_time_avg']

    ax1.scatter(success_x, success, marker='1', color='#AC3230', s=.08)
    ax2.scatter(np.arange(x_axis_len), 1. - data[sort_key]['frac_failed'], s=.1, color='#3032AC')

    plt.savefig('paper_projection.png')


if __name__ == '__main__':
    arguments = docopt(__doc__)

    if arguments['backtrack']:
        if arguments['paper']:
            plot_backtrack_paper()
        elif arguments['top1465']:
            plot_backtrack_top1465()
    elif arguments['annealing']:
        if arguments['schedules']:
            plot_schedules()
        elif arguments['geometric']:
            if arguments['paper']:
                plot_geometric_paper()
            elif arguments['top1465']:
                plot_geometric_top1465()
            else:
                print('what?')
        elif arguments['remelt']:
            if arguments['paper']:
                plot_remelt_paper()
            elif arguments['top1465']:
                plot_remelt_top1465()
            else:
                print('what?')
        else:
            print('what?')
    elif arguments['projection']:
        if arguments['paper']:
            plot_projection_paper()
        elif arguments['top1465']:
            plot_projection_top1465()
        else:
            print('what?')
    else:
        print('what?')
