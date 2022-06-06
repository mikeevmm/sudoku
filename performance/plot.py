#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Plot all the things!

Usage:
    plot backtrack paper
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

    plt.scatter(np.arange(data['puzzle'].shape[0]), data['ms_time_avg'])
    plt.plot()

if __name__ == '__main__':
    arguments = docopt(__doc__)

    if arguments['backtrack'] and arguments['paper']:
        plot_backtrack_paper()
