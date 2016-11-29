import sys
from flask import Flask, request, redirect
import twilio.twiml
from legendlib import themeSongGenerate

app = Flask(__name__)

@app.route("/monkey", methods=['GET', 'POST'])
def hello_monkey():
  """Respond to incoming calls with a simple text message."""
  resp = twilio.twiml.Response()
  resp.message("Hello, Mobile Monkey")
  return str(resp)
  
@app.route("/beep")
def beep():
    return "Beep!"

@app.route("/version")
def version():
    return sys.version

@app.route("/")
def hello():
    return "Hello World!"

@app.route("/themesong")
def theme():
    return themeSongGenerate(5)

try:
    if __name__ == "__main__":
        app.run(debug=True)
except:
    cgitb.handler()

