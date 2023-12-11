#!/usr/bin/env python3
import ssl
import http.server

class RequestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory='./dist', **kwargs)
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        http.server.SimpleHTTPRequestHandler.end_headers(self)

server_address = ('localhost', 8080)
httpd = http.server.HTTPServer(server_address, RequestHandler)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain('localhost.pem')
httpd.socket = context.wrap_socket(httpd.socket, server_side=True)
print("Serving at:", server_address)
httpd.serve_forever()
