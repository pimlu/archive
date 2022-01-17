```
openssl req -newkey rsa:2048 -nodes -keyout cert.key \
                   -x509 -out cert.pem -subj '/CN=Test Certificate' \
                   -addext "subjectAltName = DNS:localhost"

openssl x509 -inform pem -in cert.pem -outform der -out cert.der
openssl rsa -inform pem -in cert.key -outform der -out key.der


openssl x509 -pubkey -noout -in cert.pem |
                   openssl rsa -pubin -outform der |
                   openssl dgst -sha256 -binary | base64
```