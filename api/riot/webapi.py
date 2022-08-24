import cassiopeia as cass
from api.riot.masteries import MasteryDelegate


class WebApi:
    Mastery: MasteryDelegate

    def __init__(self, summoner_name):
        self._config()
        self.summoner = cass.get_summoner(
            name=summoner_name, region=cass.Region.europe_west
        )
        self.Mastery = MasteryDelegate(self.summoner)

    def _config(self):
        config = cass.get_default_config()
        key = [
            line.split("=")[1]
            for line in open("config.cfg", mode="r").readlines()
            if line.startswith("RIOT_KEY")
        ][0].strip()
        config["pipeline"]["RiotAPI"]["api_key"] = key
        config["logging"]["print_calls"] = False
        cass.apply_settings(config)
