import api.client.api_constants as ac


class LootDelegate:
    def __init__(self, session, base_url):
        # Get loot
        resp = session.get(base_url + "lol-loot/v1/player-loot")
        self._loot = resp.json()

        # categories
        # categories = set(d[ac.CATEGORIES_FIELD] for d in loot)

        # for c in categories:
        # filtered_loot = [l for l in loot if l["displayCategories"] == c]
        # print(c)
        # for fl in filtered_loot:
        # print(" ", fl["type"], fl["lootId"], fl["itemDesc"])

        # all data has same keys
        # keys = set(k for d in loot for k in d.keys())

        # for k in keys:
        #     values = set(d[k] for d in loot)

        #     if len(values) < 10:
        #         print(k)
        #         print(values)
        #         print()

    def get_credits(self):
        pass

    def get_eternals(self):
        eternals = [l for l in self._loot if l[ac.CATEGORIES_FIELD] == "ETERNALS"]
        return [e["localizedName"] for e in eternals]

    def get_mastery_tokens(self, upgradable_only=False):
        pass

    def get_openable_chest_count(self):
        pass

    def get_convertable_blue_essence(self):
        pass

    def get_missing_champ_shards(lselfoot):
        pass

    def get_owned_skin_shards(self):
        pass

    def get_interesting_skin_shards(self):
        pass
