#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Plot all the things!

Usage:
    plot backtrack paper
    plot backtrack top1465
    plot (--help|-h)

Options:
    --help      Show this screen
"""

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

    plt.axhline(np.average(data['ms_time_avg']), color='#D23D3D', label='average', linestyle='--')
    plt.axhline(np.average(p95), color='#3ED53D', label='P95 average', linestyle='--')
    plt.axhline(np.average(p50), color='#3C3DD5', label='P50 average', linestyle='--')

    plt.xticks([])
    plt.ylabel('Time (ms, average over 4 runs)')
    plt.legend(loc='center right')
    plt.savefig('backtrack_paper.png')


def plot_backtrack_top1465():
    data = np.loadtxt('backtrack.top1465.log', delimiter='\t',
            dtype={'names': ('puzzle', 'frac_failed', 'ms_time_avg'),
                     'formats': ('i1', 'f4', 'f4')})

    sort_key = np.lexsort((data['frac_failed'], data['ms_time_avg']))
    x = np.arange(data['puzzle'].shape[0])

    non_failed = data['ms_time_avg'][data['frac_failed'] < 1.]
    non_failed_key = np.argsort(-non_failed)
    non_failed_x = np.linspace(0, len(x), len(non_failed))
    
    fig, (ax1, ax2) = plt.subplots(1, 2, gridspec_kw={'width_ratios': [5, 3]})
    fig.subplots_adjust(wspace=0)

    ax1.scatter(non_failed_x, non_failed[non_failed_key], marker='1')
    ax1.set_ylabel('Run time (ms, average over 4 iterations)')
    ax1.set_xticks([])

    total_frac_failed = len(data['frac_failed'] > 0.999) / len(data['frac_failed'])
    ax2.bar(0, total_frac_failed, width=1., label='fraction failed')
    ax2.bar(0, 1. - total_frac_failed, bottom=total_frac_failed, width=1., label='fraction succeeded')
    ax2.set_xlim(-3., 3.)
    ax2.legend(loc='center right')

    ax2.axis('off')

    plt.savefig('backtrack_top1465.png')

if __name__ == '__main__':
    arguments = docopt(__doc__)

    if arguments['backtrack']:
        if arguments['paper']:
            plot_backtrack_paper()
        elif arguments['top1465']:
            plot_backtrack_top1465()
