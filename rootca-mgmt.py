# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "pyopenssl"
#
# ]
# ///

import os
import sys
import subprocess
import json
import base64
import shutil


import OpenSSL
import OpenSSL.crypto
import OpenSSL.SSL

# Simple Encode
def se(str_value):
  return base64.b64encode( bytes(b-1 for b in str_value.encode('utf-8')) ).decode('utf-8')

# Simple Decode
def sd(str_value):
  return bytes(b+1 for b in base64.b64decode(str_value.encode('utf-8'))).decode('utf-8')

def pki_and_cert_gen(config, priv_key_file, cert_file):
  k = OpenSSL.crypto.PKey()
  k.generate_key(OpenSSL.crypto.TYPE_RSA, 4096)
  # create a self-signed cert
  cert = OpenSSL.crypto.X509()
  cert.get_subject().C =            config['countryName']
  cert.get_subject().ST =           config['stateOrProvinceName']
  cert.get_subject().L =            config['localityName']
  cert.get_subject().O =            config['organizationName']
  cert.get_subject().OU =           config['organizationUnitName']
  cert.get_subject().CN =           config['commonName']
  cert.get_subject().emailAddress = config['emailAddress']
  cert.set_serial_number(      int(config['serialNumber']) )
  cert.gmtime_adj_notBefore(   int(config['validityStartInSeconds']) )
  cert.gmtime_adj_notAfter(    int(config['validityEndInSeconds']) )
  cert.set_issuer(cert.get_subject())
  cert.set_pubkey(k)
  cert.sign(k, 'sha512')

  with open(cert_file, 'w') as f:
    f.write(OpenSSL.crypto.dump_certificate(OpenSSL.crypto.FILETYPE_PEM, cert).decode("utf-8"))

  with open(priv_key_file, 'w') as f:
    f.write(OpenSSL.crypto.dump_privatekey(OpenSSL.crypto.FILETYPE_PEM, k).decode("utf-8"))

def create_pfx(crt_path, key_path, pfx_path):
    if shutil.which('openssl'):
      subprocess.run([
        'openssl', 'pkcs12', '-export', '-out', pfx_path, '-inkey', key_path, '-in', crt_path, '-passout', 'pass:'
      ], check=True)
      print(f"PFX file written to: {pfx_path}")
    else:
      # TODO we know thisis broken

      # Load the certificate
      with open(crt_path, "rb") as f:
          cert = OpenSSL.crypto.load_certificate(OpenSSL.crypto.FILETYPE_PEM, f.read())

      # Load the private key
      with open(key_path, "rb") as f:
          key = OpenSSL.crypto.load_privatekey(OpenSSL.crypto.FILETYPE_PEM, f.read())

      # Create PKCS#12 object
      p12 = OpenSSL.crypto.PKCS12()
      p12.set_certificate(cert)
      p12.set_privatekey(key)

      # Export to .pfx (password-protected)
      with open(pfx_path, "wb") as f:
          f.write(p12.export())

      print(f"PFX file written to: {pfx_path}")



repo_dir = os.path.dirname(__file__)

rootca_folder = os.path.join(repo_dir, 'rootca')

os.makedirs(rootca_folder, exist_ok=True)

rootca_priv_key_file = os.path.join(rootca_folder, 'rootca_key.key')
rootca_cert_file = os.path.join(rootca_folder, 'rootca.crt')
rootca_pfx_file = os.path.join(rootca_folder, 'rootca.pfx')
if not os.path.exists(rootca_priv_key_file):
  # This isn't supposed to be secure, it's an anti-bot-parsing technique.
  rootca_priv_key_config = {
    'countryName':            sd('VFI='),
    'stateOrProvinceName':    sd('VWhxZmhtaGA='),
    'localityName':           sd('RXFkY2RxaGJqcmF0cWY='),
    'organizationName':       sd('RXRrayxCcWhyaHIsUW5uc0JA'),
    'organizationUnitName':   sd('RXRrayxCcWhyaHIsUW5uc0JA'),
    'commonName':             sd('ZXRrayxicWhyaHItaWxiYHNkZHEtYm5s'),
    'emailAddress':           sd('ZXRrayxicWhyaHIscW5uc2JgP2lsYmBzZGRxLW92'),
    'serialNumber':           0,
    'validityStartInSeconds': 0,
    'validityEndInSeconds':   10*365*24*60*60,
  }
  input(f'Creating {rootca_priv_key_file} with the following config. Press enter to continue, or ctrl+c to fix the config.\n{json.dumps(rootca_priv_key_config, indent=4)}')

  pki_and_cert_gen(rootca_priv_key_config, rootca_priv_key_file, rootca_cert_file)

if not os.path.exists(rootca_pfx_file):
  # Generate off rootca_cert_file and rootca_priv_key_file
  create_pfx(rootca_cert_file, rootca_priv_key_file, rootca_pfx_file)


# Still TODO: Generate an intermediate cert, sign by ^^ rootca, etc and use intermediate to sign binaries.
#             For now, rootca == signing key because that's simple!
#             As long as we're blowing holes in a security model let's make them BIG HOLES!








