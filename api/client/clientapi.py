from api.client.loot import LootDelegate
import requests


class ClientApi:
    Loot: LootDelegate

    def __init__(self):
        session, base_url = self._setup_session()
        self.Loot = LootDelegate(session, base_url)

    def _setup_session(self):
        base_url, auth = self._get_client_creds()
        sess = requests.Session()
        sess.verify = "riotgames.pem"
        sess.auth = auth
        return sess, base_url

    def _get_client_creds(self):
        league_path = [
            line.split("=")[1]
            for line in open("config.cfg", mode="r").readlines()
            if line.startswith("LEAGUE_PATH")
        ][0][:-1]
        with open(league_path + "lockfile", mode="r") as f:
            info = f.readline()

        _, _, port, pwd, prot = info.split(":")
        auth = ("riot", pwd)
        base_url = f"{prot}://127.0.0.1:{port}/"
        return (base_url, auth)
