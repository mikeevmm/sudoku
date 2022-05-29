# Generates the ../reanneal.schedule file.
# This schedule is meant to be used with a glassy state as a starting point.

import numpy as np
import matplotlib.pyplot as plt
import os

if __name__ == '__main__':
    time = np.linspace(0., 1., 300)

    remelts = (np.sin(1./(1. - time**(.1) + .04) * 3) + 1.) / 2.
    #plt.plot(time, remelts)

    iterations = (1. - (1. - time)**2) * 1000
    plt.plot(time, iterations)

    temperature = (1. - time)**3 * 10
    plt.plot(time, temperature)
    plt.show()

    here = os.path.dirname(os.path.realpath(__file__))
    np.savetxt(
        f'{here}/../reanneal.schedule',
        np.array([temperature, iterations]).T,
        ['%f', '%d'],
        header='Temperature & iterations')
