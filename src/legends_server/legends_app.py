import sys
from flask import Flask
app = Flask(__name__)

@app.route("/beep")
def beep():
    return "Beep!"

@app.route("/version")
def version():
    return sys.version

@app.route("/")
def hello():
    return "Hello World!"

try:
    if __name__ == "__main__":
        app.run()
except:
    cgitb.handler()

