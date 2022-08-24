import cassiopeia as cass
from collections import defaultdict


class MasteryDelegate:
    _masteries: list[cass.ChampionMastery]

    def __init__(self, summoner: cass.Summoner) -> None:
        self._masteries = summoner.champion_masteries

    def mastery_4(self):
        return {
            c.champion.name: c.points_until_next_level
            for c in self._masteries
            if c.level == 4
        }

    def mastery_counts(self):
        counts = defaultdict(lambda: 0)
        for cm in self._masteries:
            counts[cm.level] += 1
        return dict(counts)

    def mastery_tokens(self):
        tokens = [cm for cm in self._masteries if cm.level in [5, 6]]
        available = [c for c in tokens if c.tokens < (c.level - 3)]
        return {c.champion.name: c.tokens for c in available}

    def mastery_chests(self, min_level=None):
        chests_avail = [
            c
            for c in self._masteries
            if not c.chest_granted and (min_level is None or c.level >= min_level)
        ]
        chests_avail.sort(key=lambda c: c.level, reverse=True)
        return [c.champion.name for c in chests_avail]

    def mastery_unplayed(self):
        return [c.champion.name for c in self._masteries if c.level == 0]
