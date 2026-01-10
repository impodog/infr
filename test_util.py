import json

import requests

base = "http://127.0.0.1:4321/"


def work(func, url, single=None, **kwargs):
    if single is None:
        return func(base + url, json=kwargs)
    else:
        return func(base + url, json=single)


def get(*args, **kwargs):
    return work(requests.get, *args, **kwargs)


def post(*args, **kwargs):
    return work(requests.post, *args, **kwargs)


def load_scripts(*args):
    for script in args:
        post("scripts/add", name=script, path="scripts/" + script.lower() + ".lua")
    post("scripts/reload")


def load_session(path):
    id = post("session/load", path=path).json()["id"]
    print("id is", id)
    post("session/import", id)


def reload_scripts(id):
    post("scripts/reload")
    post("session/import", id)


def step(id, direction):
    return post("session/step", session_id=id, direction=direction)


def get_map(id):
    return get("session/map", id)


def init():
    load_scripts("Basic")
    load_session("scripts/level.toml")


"""
load_scripts("Basic")
load_session("scripts/level.toml")
"""
