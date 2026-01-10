import requests

base = "http://127.0.0.1:4321/"


def get(url, *args, **kwargs):
    return requests.get(base + url, args, **kwargs)


def post(url, *args, **kwargs):
    return requests.post(base + url, args, **kwargs)
