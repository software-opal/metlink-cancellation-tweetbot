import requests as r  # Keyboard's q key doesn't work.
import pprint


SERVICE_ALERTS = "https://api.opendata.metlink.org.nz/v1/gtfs-rt/servicealerts"


EFFECT_CAUSE_SKIP_LIST = {
    ("ACCESSIBILITY_ISSUE", "CONSTRUCTION"),
    ("STOP_MOVED", "CONSTRUCTION")
}


def get_en_translation(object):
    if object:
        for translation in object["translation"]:
            if translation["language"] == "en":
                return translation["text"]
    return None


def main():
    with r.get(
        SERVICE_ALERTS,
        headers={"x-api-key": "iuoMNXQjzC1PjijgMjKkHhYWPb4ZES2UpaYfgsd0"},
    ) as res:
        res.raise_for_status()
        alerts = res.json()

    entities = alerts["entity"]
    for entity in entities:
        alert = entity["alert"]
        effect, cause = alert["effect"], alert["cause"]

        new_alert = {
            "id": entity["id"],
            "timestamp": entity["timestamp"],
            **alert,
            "description_text": get_en_translation(alert["description_text"]),
            "header_text": get_en_translation(alert["header_text"]),
            "url": get_en_translation(alert.get("url")),
        }

        if (effect, cause) not in EFFECT_CAUSE_SKIP_LIST:
            pprint.pprint(new_alert)


if __name__ == "__main__":
    main()
