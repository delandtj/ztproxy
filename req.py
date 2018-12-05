import requests,json
req = """
{
  "auth": null,
  "name": "testnet6",
  "private": 1,
  "allowPassiveBridging": 0,
  "v4AssignMode": "zt",
  "v6AssignMode": "none",
  "routes": [ { "target": "10.149.0.0/24", "via": null, "flags": 0, "metric": 0 } ],
  "ipAssignmentPools": [ { "ipRangeStart": "10.149.0.10", "ipRangeEnd": "10.149.0.250" } ],
 "rules": [
  {
   "etherType": 2048,
   "not": true,
   "or": false,
   "type": "MATCH_ETHERTYPE"
  },
  {
   "etherType": 2054,
   "not": true,
   "or": false,
   "type": "MATCH_ETHERTYPE"
  },
  {
   "etherType": 34525,
   "not": true,
   "or": false,
   "type": "MATCH_ETHERTYPE"
  },
  {
   "not": false,
   "or": false,
   "type": "ACTION_DROP"
  },
  {
   "not": false,
   "or": false,
   "type": "ACTION_ACCEPT"
  }
 ],
 "capabilities": [],
 "tags": []
}
"""

# pubsecret = '/var/lib/zerotier-one/authtoken.secret'
pubsecret = '/home/delandtj/.zeroTierOneAuthToken'

with open(pubsecret, 'r') as f:
    authtoken = f.read().strip()
with open ('/var/lib/zerotier-one/identity.public','r') as f:
    serverid = f.read().split(':')[0]

url = "http://localhost:9993/controller/network/%s______?auth=%s" % (serverid,authtoken)

resp = requests.post(url,json=json.loads(req))
resp
resp.content
