from pprint import pprint
from api.riot.webapi import WebApi
from api.client.clientapi import ClientApi
import api.client.api_constants as ac


_webapi = WebApi("Plexian", "EUW")
_clientapi = ClientApi()


def print_eternals():
    data = _clientapi.Loot.get_eternals()
    pprint(data)


def print_mastery_tokens():
    data = _clientapi.Loot.get_mastery_tokens()

    data.sort(
        key=lambda d: (
            -d["level"],
            -d[ac.FIELD_COUNT],
            d["upgradable"],
            d[ac.FIELD_ITEMDESC],
        )
    )

    output = ""
    for d in data:
        champ = d[ac.FIELD_ITEMDESC]
        level = d["level"]
        count = d[ac.FIELD_COUNT]
        required = d["req_count"]
        text = f"{champ} (Level {level}): {count}/{required} tokens"

        upgradable = d["upgradable"]
        if upgradable is not None:
            text += " - READY FOR UPGRADE" if upgradable else " - MISSING SHARD"

        output += text + "\n"

    print(output)


def print_blue_essence():
    data = _clientapi.Loot.get_convertable_blue_essence()
    be = _clientapi.Loot.get_credits()[ac.CURRENCY_BLUE_ESSENCE]
    total, keep1, keep2 = [data[k] for k in ["total", "keep_1", "keep_2"]]

    print(f"Current BE: {be}")
    print(f"Convertable BE: {total}")
    print(f"Convertable BE (Keep one shard per champ): {keep1}")
    print(f"Convertable BE (Keep two shards per champ): {keep2}")


def print_missing_champ_shards():
    all_champs = _webapi.Dragon.get_champs().values()
    shards = _clientapi.Loot.get_missing_champ_shards(all_champs)
    print(shards)


def print_interesting_skins():
    # Get champs, enhance with champ name, sort by mastery
    sorted_champs = _webapi.Mastery.champs_sorted_by_mastery()
    champs_dict = _webapi.Dragon.get_champs()
    skin_shards = _clientapi.Loot.get_owned_skin_shards()
    skin_shards = [
        dict(**s, champ=champs_dict[s[ac.FIELD_PARENTID]]) for s in skin_shards
    ]

    # Filter and sort
    played_champs = _webapi.Mastery.champs_with_minpts(10000)
    skin_shards = [s for s in skin_shards if s["champ"] in played_champs]
    skin_shards.sort(key=lambda s: sorted_champs.index(s["champ"]))

    # Print nicely
    for shard in skin_shards:
        print(shard["champ"] + ": " + shard[ac.FIELD_ITEMDESC])
