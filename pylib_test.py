#!/usr/bin/env python
# coding: utf-8

import sys, ctypes
from ctypes import c_void_p, c_uint8, c_uint32, c_char_p, CFUNCTYPE, POINTER, WINFUNCTYPE

prefix = {'win32': ''}.get(sys.platform, 'lib')
extension = {'darwin': '.dylib', 'win32': '.dll'}.get(sys.platform, '.so')
lib = ctypes.cdll.LoadLibrary(prefix + "legendsof1f600" + extension)

lib.theme_song_generate.argtypes = (c_uint8, )
lib.theme_song_generate.restype = c_void_p

lib.theme_song_free.argtypes = (c_void_p, )

def themeSongGenerate(count):
  ptr = lib.theme_song_generate(count)
  try:
    return ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
  finally:
    lib.theme_song_free(ptr)

print themeSongGenerate(8)

lib.how_many_characters.argtypes = (c_char_p,)
lib.how_many_characters.restype = c_uint32

print lib.how_many_characters("Windows doesn't like unicode.".encode('utf-8'))


#lib.process_command.argtypes = (c_char_p, POINTER(c_char_p))
lib.process_command.restype = None

CB = CFUNCTYPE(c_uint8, POINTER(c_char_p))

def py_cb(result):
  print ctypes.cast(result, ctypes.c_char_p).value.decode('utf-8')
  return 0

cb_func = CB(py_cb)

def processCommand(cmd, callback):
  lib.process_command(cmd.encode('utf-8'), callback)

processCommand("Holla back!", cb_func)
