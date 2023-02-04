import itertools
from functools import reduce

# TODO: randomize number of stocks and outcomes to run a simulation and keep
# track of biggest number of non-unique outcomes from the return perspective
def print_unique_outcomes():
    # E.g. 5 stocks, each with 3 outcomes
    outcomes_for_each_stock = [
        [-50, 0, 100],
        [-50, 0, 100],
        [-100, 0, 200],
        [-50, 0, 100],
        [-50, 0, 100],
    ]
    
    outer_product = list(itertools.product(*outcomes_for_each_stock))
    print(f"Number of all combinations: {len(outer_product)}")
    
    unique = list(set(map(lambda p: tuple(sorted(p)), outer_product)))
    print(f"Number of combinations having the same outcome: {len(unique)}")


def print_total_number_of_bets():
    # Assuming we'll stop investing in our 80s
    n_years_to_invest = 50

    # An estimate assuming that:
    # 1. We'll keep adding money to the portfolio 4 times a year for e.g. 20
    #    more years, which will cause us to effectivelly re-allocate
    # 2. We'll find a new idea to invest in once per year
    n_rebalancings_per_year = 2

    print(f"Total number of allocations: {n_years_to_invest * n_rebalancings_per_year}")


if __name__ == "__main__":
    print_unique_outcomes()
    print_total_number_of_bets()
