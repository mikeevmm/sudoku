# Generates the ../remelt.schedule file.

import numpy as np
import matplotlib.pyplot as plt
import os

if __name__ == '__main__':
    time = np.linspace(0., 1., 300)

    remelts = (np.sin(1./(1. - time**(.1) + .04) * 3) + 1.) / 2.
    #plt.plot(time, remelts)

    iterations = np.exp(time**2 * np.log(100)) * 100
    #plt.plot(time, iterations)

    temperature = np.exp(-np.tan(time * 0.9 * np.pi / 2.)) * remelts
    plt.plot(time, temperature)
    plt.show()

    here = os.path.dirname(os.path.realpath(__file__))
    np.savetxt(
        f'{here}/../remelt.schedule',
        np.array([temperature, iterations]).T,
        ['%12g', '%d'],
        header='Temperature & iterations')
