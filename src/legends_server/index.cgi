#!/usr/bin/python3.4
import cgitb; cgitb.enable()
from wsgiref.handlers import CGIHandler
from legends_app import app

class ScriptNameStripper(object):
    def __init__(self, app):
        self.app = app
    def __call__(self, environ, start_response):
        environ['SCRIPT_NAME'] = ''
        return self.app(environ, start_response)

app = ScriptNameStripper(app)

try:
    CGIHandler().run(app)
except:
    cgitb.handler()

