import api.client.api_constants as ac
from api.client._util import simplify_dicts
import itertools as it


class LootDelegate:
    def __init__(self, session, base_url):
        # Get loot
        resp = session.get(base_url + "lol-loot/v1/player-loot")
        self._loot = resp.json()

    def get_credits(self):
        currencies = [l for l in self._loot if l[ac.FIELD_TYPE] == ac.TYPE_CURRENCY]
        money_dict = {c[ac.FIELD_LOOTNAME]: c[ac.FIELD_COUNT] for c in currencies}
        return money_dict

    def get_eternals(self):
        eternals = [
            l for l in self._loot if l[ac.FIELD_CATEGORIES] == ac.CATEGORY_ETERNALS
        ]
        return [e["localizedName"] for e in eternals]

    def get_mastery_tokens(self):
        # Champ shards
        champs = self._get_champ_shards()
        shard_dict = {c[ac.FIELD_ITEMDESC]: c[ac.FIELD_COUNT] for c in champs}

        # Tokens
        chests = [l for l in self._loot if l[ac.FIELD_CATEGORIES] == ac.CATEGORY_CHEST]
        tokens = [c for c in chests if c[ac.FIELD_TYPE] == ac.TYPE_MASTERY_TOKEN]
        simple_tokens = simplify_dicts(
            tokens, [ac.FIELD_ITEMDESC, ac.FIELD_LOOTNAME, ac.FIELD_COUNT]
        )

        for t in simple_tokens:
            level = 7 if t[ac.FIELD_LOOTNAME] == ac.TOKEN_NAME_MASTERY_SEVEN else 6
            req_count = level - 4

            upgradable = None
            if req_count == t[ac.FIELD_COUNT]:
                upgradable = False
                if t[ac.FIELD_ITEMDESC] in shard_dict:
                    if shard_dict[t[ac.FIELD_ITEMDESC]] > 0:
                        upgradable = True

            t.update({"level": level, "req_count": req_count, "upgradable": upgradable})

        return simple_tokens

    def get_openable_chest_count(self):
        # Getting keys unknown
        pass

    def get_convertable_blue_essence(self):
        champs = self._get_champ_shards()
        total = sum(c[ac.FIELD_COUNT] * c[ac.FIELD_DISENCHANT] for c in champs)
        keep1 = sum(
            max(c[ac.FIELD_COUNT] - 1, 0) * c[ac.FIELD_DISENCHANT] for c in champs
        )
        keep2 = sum(
            max(c[ac.FIELD_COUNT] - 2, 0) * c[ac.FIELD_DISENCHANT] for c in champs
        )
        return {"total": total, "keep_1": keep1, "keep_2": keep2}

    def get_missing_champ_shards(self, all_champs):
        shards = self._get_champ_shards()
        owned = set(s[ac.FIELD_ITEMDESC] for s in shards)
        return sorted(list(set(all_champs) - owned))

    def get_owned_skin_shards(self):
        skins = [l for l in self._loot if l[ac.FIELD_CATEGORIES] == ac.CATEGORY_SKINS]
        return simplify_dicts(skins, [ac.FIELD_ITEMDESC, ac.FIELD_PARENTID])

    def _get_champ_shards(self):
        champs = [
            l for l in self._loot if l[ac.FIELD_CATEGORIES] == ac.CATEGORY_CHAMPION
        ]
        return simplify_dicts(
            champs,
            [ac.FIELD_ITEMDESC, ac.FIELD_COUNT, ac.FIELD_DISENCHANT],
        )
