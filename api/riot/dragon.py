import cassiopeia as cass


class DragonDelegate:
    def __init__(self, region):
        self.region = region
        pass

    def get_champs(self):
        return {c.id: c.name for c in cass.get_champions(self.region)}
