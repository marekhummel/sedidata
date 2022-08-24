from api.client.clientapi import ClientApi
from api.riot.webapi import WebApi
from pprint import pprint

SUMMONER = "Plexian"

# webapi = WebApi(SUMMONER)
# pprint(webapi.Mastery.mastery_4())
# pprint(webapi.Mastery.mastery_tokens())
# pprint(webapi.Mastery.mastery_chests(7))
# pprint(webapi.Mastery.mastery_counts())
# pprint(webapi.Mastery.mastery_unplayed())


clientapi = ClientApi()
pprint(clientapi.Loot.get_eternals())
